import * as anchor from "@coral-xyz/anchor";
import { BN, Program } from "@coral-xyz/anchor";
import { SolVsEth } from "../target/types/sol_vs_eth";
import { Keypair, PublicKey } from "@solana/web3.js";
import { TOKEN_PROGRAM_ID, createAccount, createMint, mintTo } from "@solana/spl-token";


const GLOBAL_STATE_SEED = "global-state";
const GLOBAL_AUTH_SEED = "global-auth";
let payer = new anchor.web3.Keypair();
let globalState: PublicKey;
let globalAuthPda: PublicKey;
let houseWallet: PublicKey;
let bettingToken: PublicKey;
let gameVault: PublicKey;
let bettingGameAddress: PublicKey;
let userTokenAccount: PublicKey;


const ETH_ORACLE = "JBu1AL4obBcCMqKBBxhpWCNUt136ijcuMZLFvTP7iWdB";
const SOL_ORACLE = "H6ARHf6YXhGYeQfUzQNGk6rDNnLBQKrenN712K4AQJEG";


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
    userTokenAccount = await createAccount(connection, payer, testToken, program.provider.publicKey);

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

    await mintTo(connection, payer, testToken, userTokenAccount, payer, 1000000000);
    await mintTo(connection, payer, testToken, houseWallet, payer, 1000000000);

  });
  it("change global state", async () => {
    const tx = await program.methods.changeGlobalState(new BN(500), new BN(0.5 * 1e6), new BN(10), new BN(10)).accounts({
      signer: program.provider.publicKey,
      globalState,
      newCrankAdmin: program.provider.publicKey,
    }).rpc(OPTS);
  });
  it("creating a new game", async () => {

    const bettingGameAddress_ = new Keypair();
    bettingGameAddress = bettingGameAddress_.publicKey;

    const bettingVault_ = new Keypair();

    gameVault = bettingVault_.publicKey;
    const tx = await program.methods.startGame().accounts({
      signer: program.provider.publicKey,
      game: bettingGameAddress,
      bettingToken: bettingToken,
      gameVault,
      globalAuthPda,
      globalState,
      systemProgram: anchor.web3.SystemProgram.programId,
      tokenProgram: TOKEN_PROGRAM_ID,

    }).signers([bettingVault_, bettingGameAddress_]).rpc(OPTS);


  });


  it("betting on a game", async () => {


    const tx = await program.methods.placeBet(new BN(1000), 0).accounts({
      signer: program.provider.publicKey,
      game: bettingGameAddress,
      bettingToken,
      globalAuthPda,
      globalState,
      gameVault,
      houseWallet,
      payer: userTokenAccount,
      systemProgram: anchor.web3.SystemProgram.programId,
      tokenProgram: TOKEN_PROGRAM_ID,


    }).rpc(OPTS);
  });

  it("betting on the other side", async () => {
    try {
      const tx = await program.methods.placeBet(new BN(1000), 1).accounts({
        signer: program.provider.publicKey,
        game: bettingGameAddress,
        bettingToken,
        globalAuthPda,
        globalState,
        gameVault,
        houseWallet,
        payer: userTokenAccount,
        systemProgram: anchor.web3.SystemProgram.programId,
        tokenProgram: TOKEN_PROGRAM_ID,
      }).rpc(OPTS);
      console.log(tx)
      throw Error("Betting on 2 different sides on the same game");
    }
    catch (e) { }
  });

  it("start anticipation", async () => {

    // wait for 10 seconds
    await new Promise((resolve) => setTimeout(resolve, 10000));
    const tx = await program.methods.startAnticipation().accounts({
      signer: program.provider.publicKey,
      game: bettingGameAddress,
      globalState,
      ethFeed: new PublicKey(ETH_ORACLE),
      solFeed: new PublicKey(SOL_ORACLE),
    }).rpc(OPTS);
    console.log(tx);
  })
  it("betting when game is closed", async () => {

    try {
      const tx = await program.methods.placeBet(new BN(1000), 0).accounts({
        signer: program.provider.publicKey,
        game: bettingGameAddress,
        bettingToken,
        globalAuthPda,
        globalState,
        gameVault,
        houseWallet,
        payer: userTokenAccount,
        systemProgram: anchor.web3.SystemProgram.programId,
        tokenProgram: TOKEN_PROGRAM_ID,
      }).rpc(OPTS);
      console.log(tx)
      throw Error("Trade went through when game is closed");
    }
    catch (e) { }
  });

  it("end anticipation", async () => {

    await new Promise((resolve) => setTimeout(resolve, 10000));

    const tx = await program.methods.resolveGame().accounts({
      signer: program.provider.publicKey,
      game: bettingGameAddress,
      globalState,
      gameVault,
      houseWallet,
      ethFeed: new PublicKey(ETH_ORACLE),
      solFeed: new PublicKey(SOL_ORACLE),
      tokenProgram: TOKEN_PROGRAM_ID,
      globalAuthPda,
    }).rpc(OPTS);
    console.log(tx);
  })

  it("claiming rewards", async () => {
    const tx = await program.methods.claimWin().accounts({
      game: bettingGameAddress,
      gameVault,
      globalAuthPda,
      globalState,
      receiver: userTokenAccount,
      signer: program.provider.publicKey,
      tokenProgram: TOKEN_PROGRAM_ID,
    }).rpc(OPTS);
  });

});
