use anchor_lang::prelude::*;
use anchor_spl::associated_token::AssociatedToken;
use anchor_spl::token::{self, Mint, Token, TokenAccount, Transfer};
declare_id!("qs6tDmJwf4wFBpDbQCDYkMQK774xf91LeKc83XieANK");

#[program]
pub mod my_program {
    use super::*;

    pub fn create_token(ctx: Context<CreateToken>, token_amount: u64) -> Result<()> {
        // 初始化 Mint 账户
        let init_mint_accounts = token::InitializeMint {
            mint: ctx.accounts.token_mint.to_account_info(),
            rent: ctx.accounts.rent.to_account_info(),
        };
        let init_mint_ctx = CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            init_mint_accounts,
        );
        token::initialize_mint(
            init_mint_ctx.with_signer(&[&[&[]]]),
            9,
            &ctx.accounts.token_authority.key(),
            Some(&ctx.accounts.token_authority.key()),
        )?;

        // 铸造代币到创建者的代币账户
        let mint_to_accounts = token::MintTo {
            mint: ctx.accounts.token_mint.to_account_info(),
            to: ctx.accounts.token_account.to_account_info(),
            authority: ctx.accounts.token_authority.to_account_info(),
        };
        let mint_to_ctx = CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            mint_to_accounts,
        );
        token::mint_to(mint_to_ctx.with_signer(&[&[&[]]]), token_amount)?;

        Ok(())
    }

    pub fn create_contract_token_account(
        ctx: Context<CreateContractTokenAccount>,
        amount: u64,
    ) -> Result<()> {
        if ctx.accounts.contract_token_account.amount == 0 {
            // 如果合约的代币账户不存在,创建它
            let cpi_accounts = anchor_spl::associated_token::Create {
                payer: ctx.accounts.user.to_account_info(),
                associated_token: ctx.accounts.contract_token_account.to_account_info(),
                authority: ctx.accounts.contract_sol_account.to_account_info(),
                mint: ctx.accounts.token_mint.to_account_info(),
                system_program: ctx.accounts.system_program.to_account_info(),
                token_program: ctx.accounts.token_program.to_account_info(),
            };
            let cpi_program = ctx.accounts.associated_token_program.to_account_info();
            let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
            anchor_spl::associated_token::create(cpi_ctx)?;
        }

        // 将代币从用户账户转移到合约账户
        let cpi_accounts = Transfer {
            from: ctx.accounts.user_token_account.to_account_info(),
            to: ctx.accounts.contract_token_account.to_account_info(),
            authority: ctx.accounts.user.to_account_info(),
        };
        let cpi_program = ctx.accounts.token_program.to_account_info();
        let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
        token::transfer(cpi_ctx, amount)?;

        Ok(())
    }
    pub fn exchange_sol_for_tokens(ctx: Context<ExchangeSolForTokens>, amount: u64) -> Result<()> {
        let sol_transfer_amount = amount;
        let token_transfer_amount = amount * 10; // Assuming a 1:10 exchange rate

        // Transfer SOL from user to contract
        anchor_lang::solana_program::program::invoke(
            &anchor_lang::solana_program::system_instruction::transfer(
                &ctx.accounts.user_sol_account.key(),
                &ctx.accounts.contract_sol_account.key(),
                sol_transfer_amount,
            ),
            &[
                ctx.accounts.user_sol_account.to_account_info(),
                ctx.accounts.contract_sol_account.to_account_info(),
            ],
        )?;

        // Transfer SPL tokens from contract to user
        let seeds = &[
            b"token_account".as_ref(),
            &[ctx.bumps.contract_token_account],
        ];
        let signer = &[&seeds[..]];
        let cpi_accounts = Transfer {
            from: ctx.accounts.contract_token_account.to_account_info(),
            to: ctx.accounts.user_token_account.to_account_info(),
            authority: ctx.accounts.contract_sol_account.to_account_info(),
        };
        let cpi_program = ctx.accounts.token_program.to_account_info();
        let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, signer);
        token::transfer(cpi_ctx, token_transfer_amount)?;

        Ok(())
    }
    pub fn withdraw_sol(ctx: Context<WithdrawSol>, amount: u64) -> Result<()> {
        let contract_sol_account = &mut ctx.accounts.contract_sol_account;
        let user_sol_account = &mut ctx.accounts.user_sol_account;

        **contract_sol_account.try_borrow_mut_lamports()? -= amount;
        **user_sol_account.try_borrow_mut_lamports()? += amount;

        Ok(())
    }
}

#[derive(Accounts)]
pub struct CreateToken<'info> {
    #[account(mut)]
    pub token_authority: Signer<'info>,
    #[account(
        init,
        payer = token_authority,
        mint::decimals = 9,
        mint::authority = token_authority,
    )]
    pub token_mint: Account<'info, Mint>,
    #[account(
        init,
        payer = token_authority,
        associated_token::mint = token_mint,
        associated_token::authority = token_authority,
    )]
    pub token_account: Account<'info, TokenAccount>,
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

#[derive(Accounts)]
pub struct CreateContractTokenAccount<'info> {
    #[account(mut)]
    pub user: Signer<'info>,
    #[account(mut)]
    pub user_token_account: Account<'info, TokenAccount>,
    pub contract_sol_account: SystemAccount<'info>,
    #[account(
        init_if_needed,
        payer = user,
        associated_token::mint = token_mint,
        associated_token::authority = contract_sol_account,
    )]
    pub contract_token_account: Account<'info, TokenAccount>,
    pub token_mint: Account<'info, token::Mint>,
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct ExchangeSolForTokens<'info> {
    #[account(mut)]
    pub user_sol_account: Signer<'info>,
    #[account(mut)]
    pub user_token_account: Account<'info, TokenAccount>,
    #[account(mut)]
    pub contract_sol_account: SystemAccount<'info>,
    #[account(
        mut,
        seeds = [b"token_account"],
        bump,
    )]
    pub contract_token_account: Account<'info, TokenAccount>,
    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
pub struct WithdrawSol<'info> {
    #[account(mut)]
    pub user_sol_account: SystemAccount<'info>,
    #[account(mut)]
    pub contract_sol_account: SystemAccount<'info>,
}
