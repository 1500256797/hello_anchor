import * as anchor from '@coral-xyz/anchor'
import { Program } from '@coral-xyz/anchor'
import { HelloAnchor } from '../target/types/hello_anchor'

describe('hello_anchor', () => {
	// Configure the client to use the local cluster.
	anchor.setProvider(anchor.AnchorProvider.env())

	const program = anchor.workspace.HelloAnchor as Program<HelloAnchor>

	it('setup game!', async () => {
		const gameKeypair = anchor.web3.Keypair.generate()
		const playerOne = (program.provider as anchor.AnchorProvider).wallet
		const playerTwo = anchor.web3.Keypair.generate()
		await program.methods
			.setupGame(playerTwo.publicKey)
			.accounts({
				game: gameKeypair.publicKey,
				playerOne: playerOne.publicKey,
			})
			.signers([gameKeypair])
			.rpc()

		let gameState = await program.account.game.fetch(gameKeypair.publicKey)
		expect(gameState.turn).to.equal(1)
		expect(gameState.players).to.eql([
			playerOne.publicKey,
			playerTwo.publicKey,
		])
		expect(gameState.state).to.eql({ active: {} })
		expect(gameState.board).to.eql([
			[null, null, null],
			[null, null, null],
			[null, null, null],
		])
	})
})
