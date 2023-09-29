import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { SolVsEth } from "../target/types/sol_vs_eth";
import { Keypair } from "@solana/web3.js";
import { TOKEN_PROGRAM_ID, createAccount, createMint } from "@solana/spl-token";


const GLOBAL_STATE_SEED = "global_state";
const GLOBAL_AUTH_SEED = "global-auth";
let payer = new anchor.web3.Keypair();

const OPTS: anchor.web3.ConfirmOptions = {
  skipPreflight: true,
};
describe("sol-vs-eth", () => {
  // Configure the client to use the local cluster.
  anchor.setProvider(anchor.AnchorProvider.env());

  const program = anchor.workspace.SolVsEth as Program<SolVsEth>;

  it("initializing glabal state!", async () => {

    await program.provider.connection.confirmTransaction(await program.provider.connection.requestAirdrop(payer.publicKey, 10_000_000_000));
    const connection = program.provider.connection;

    const [globalAuth, globalAuthBump] = await anchor.web3.PublicKey.findProgramAddress(
      [Buffer.from(GLOBAL_AUTH_SEED)],
      program.programId
    );
    const [globalState, globalStateBump] = await anchor.web3.PublicKey.findProgramAddress(
      [Buffer.from(GLOBAL_STATE_SEED)],
      program.programId
    );

    const testToken = await createMint(connection, payer, payer.publicKey, payer.publicKey, 6);
    const houseWallet = new Keypair();




    const tx = await program.methods.initialize().accounts({
      signer: payer.publicKey,
      globalState,
      globalAuthPda: globalAuth,
      houseWallet: houseWallet.publicKey,
      systemProgram: anchor.web3.SystemProgram.programId,
      bettingCurrency: testToken,
      tokenProgram: TOKEN_PROGRAM_ID,

    }).signers([houseWallet]).rpc(OPTS);
  });
});
