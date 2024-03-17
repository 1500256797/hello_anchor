use anchor_lang::prelude::*;

declare_id!("F8o19NtvNsQMWqQDCu1WfYntrjooL8e11j8b7KjNTUtu");

#[program]
pub mod hello_anchor {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize {}
