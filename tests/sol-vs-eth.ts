import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { SolVsEth } from "../target/types/sol_vs_eth";
import { Keypair, PublicKey } from "@solana/web3.js";
import { TOKEN_PROGRAM_ID, createAccount, createMint } from "@solana/spl-token";


const GLOBAL_STATE_SEED = "global-state";
const GLOBAL_AUTH_SEED = "global-auth";
let payer = new anchor.web3.Keypair();
let globalState: PublicKey;
let globalAuthPda: PublicKey;
let houseWallet: PublicKey;
let bettingToken: PublicKey;
let bettingVault: PublicKey;


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

    const [globalAuth_, globalAuthBump] = await anchor.web3.PublicKey.findProgramAddress(
      [Buffer.from(GLOBAL_AUTH_SEED)],
      program.programId
    );
    globalAuthPda = globalAuth_;
    const [globalState_, globalStateBump] = await anchor.web3.PublicKey.findProgramAddress(
      [Buffer.from(GLOBAL_STATE_SEED)],
      program.programId
    );
    globalState = globalState_;

    const testToken = await createMint(connection, payer, payer.publicKey, payer.publicKey, 6);
    bettingToken = testToken;
    const houseWallet_ = new Keypair();
    houseWallet = houseWallet_.publicKey;
    const tx = await program.methods.initialize().accounts({
      signer: program.provider.publicKey,
      globalState,
      globalAuthPda,
      houseWallet: houseWallet_.publicKey,
      bettingCurrency: testToken,
      systemProgram: anchor.web3.SystemProgram.programId,
      tokenProgram: TOKEN_PROGRAM_ID,
    }).signers([houseWallet_]).rpc(OPTS);
    console.log("tx", tx);
  });

  it("creating a new game", async () => {

    const bettingGameAddress = new Keypair();

    const bettingVault = new Keypair();

    const tx = await program.methods.startBetting().accounts({
      signer: program.provider.publicKey,
      bet: bettingGameAddress.publicKey,
      bettingToken: bettingToken,
      bettingVault: bettingVault.publicKey,
      globalAuthPda,
      globalState,
      systemProgram: anchor.web3.SystemProgram.programId,
      tokenProgram: TOKEN_PROGRAM_ID,

    }).signers([bettingVault,bettingGameAddress]).rpc(OPTS);


  });

});
