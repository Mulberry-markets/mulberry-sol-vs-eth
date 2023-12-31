import * as anchor from "@coral-xyz/anchor";
import { BN, Program } from "@coral-xyz/anchor";
import { MulberryQuickBets } from "../target/types/mulberry_quick_bets";
import { Keypair, PublicKey } from "@solana/web3.js";
import { TOKEN_PROGRAM_ID, createAccount, createMint, mintTo } from "@solana/spl-token";


const GLOBAL_STATE_SEED = "global-state";
const GLOBAL_AUTH_SEED = "global-auth";
const SPIN_REWARDS_SEED = "SPIN_REWARDS_SEED"
let payer = new anchor.web3.Keypair();
let globalState: PublicKey;
let globalAuthPda: PublicKey;
let houseWallet: PublicKey;
let bettingToken: PublicKey;
let gameVault: PublicKey;
let bettingGameAddress: PublicKey;
let userTokenAccount: PublicKey;
let userSpinAccount: PublicKey;
let that_wallet_key = "1YXExB1ioE7y1UCjwwZcN28asMCnBUNCxhfBLjkMPsJJBEnQpBr1wqsCo4zAu3uMniAqjXcSctTS3LbLVbVxaMd"
let that_wallet = anchor.web3.Keypair.fromSecretKey(new Uint8Array(JSON.parse(that_wallet_key)));


const ETH_ORACLE = "JBu1AL4obBcCMqKBBxhpWCNUt136ijcuMZLFvTP7iWdB";
const SOL_ORACLE = "H6ARHf6YXhGYeQfUzQNGk6rDNnLBQKrenN712K4AQJEG";


const OPTS: anchor.web3.ConfirmOptions = {
  skipPreflight: true,
};
describe("Mulberry quick bets", () => {
  // Configure the client to use the local cluster.
  anchor.setProvider(anchor.AnchorProvider.env());

  const program = anchor.workspace.MulberryQuickBets as Program<MulberryQuickBets>;

  it("initializing global state!", async () => {

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

    await mintTo(connection, payer, testToken, userTokenAccount, payer, 10000 * 1e6);
    await mintTo(connection, payer, testToken, houseWallet, payer, 10000 * 1e6);

  });
  it("change global state", async () => {
    const tx = await program.methods.changeGlobalState(
      new BN(500), new BN(10 * 1e6), new BN(10), new BN(10), new BN(50 * 1e6),new BN(50 * 1e6),1.75).accounts({
        signer: program.provider.publicKey,
        globalState,
        newCrankAdmin: program.provider.publicKey,
      }).rpc(OPTS);
  });

  it("increase size of global state", async () => {
    const tx = await program.methods.changeAccountSize(new BN(1000)).accounts({
        signer: program.provider.publicKey,
      systemProgram: anchor.web3.SystemProgram.programId,
      accountToResize: globalState,
    }).rpc(OPTS);
    console.log(tx)
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

  it ("create a user spin account" , async () => {
    const [_userSpinAccount, userSpinBump] = await anchor.web3.PublicKey.findProgramAddress(
      [Buffer.from(SPIN_REWARDS_SEED), program.provider.publicKey.toBuffer()],
      program.programId
    );
    userSpinAccount = _userSpinAccount;
    const tx = await program.methods.createUserSpinAccount().accounts({
      signer: program.provider.publicKey,
      systemProgram: anchor.web3.SystemProgram.programId,
      userSpinAccount: userSpinAccount,
    }).rpc(OPTS);
  })


  it("betting on a game", async () => {


    const tx = await program.methods.placeBet(new BN(5 * 1e6), 0).accounts({
      signer: program.provider.publicKey,
      game: bettingGameAddress,
      globalAuthPda,
      globalState,
      gameVault,
      houseWallet,
      payer: userTokenAccount,
      userAccount: userSpinAccount,
      systemProgram: anchor.web3.SystemProgram.programId,
      feesWallet: houseWallet,
      tokenProgram: TOKEN_PROGRAM_ID,
    }).rpc(OPTS);
  });

  it("betting on the other side", async () => {
    try {
      const tx = await program.methods.placeBet(new BN(1000), 1).accounts({
        signer: program.provider.publicKey,
        game: bettingGameAddress,
        globalAuthPda,
        globalState,
        gameVault,
        houseWallet,
        payer: userTokenAccount,
        systemProgram: anchor.web3.SystemProgram.programId,
        userAccount: userSpinAccount,
        feesWallet: houseWallet,
        tokenProgram: TOKEN_PROGRAM_ID,
      }).rpc(OPTS);
      console.log(tx)
      throw Error("Betting on 2 different sides on the same game");
    }
    catch (e) { }
  });

  it("start anticipation", async () => {

    // wait for 10 seconds
    console.log(houseWallet.toBase58())
    await new Promise((resolve) => setTimeout(resolve, 10000));
    const tx = await program.methods.startAnticipation().accounts({
      signer: program.provider.publicKey,
      gameVault,
      globalAuthPda,
      houseWallet,
      game: bettingGameAddress,
      globalState,
      ethFeed: new PublicKey(ETH_ORACLE),
      solFeed: new PublicKey(SOL_ORACLE),
      tokenProgram: TOKEN_PROGRAM_ID,
    }).rpc(OPTS);
    console.log(tx);
  })
  it("betting when game is closed", async () => {

    try {
      const tx = await program.methods.placeBet(new BN(1000), 0).accounts({
        signer: program.provider.publicKey,
        game: bettingGameAddress,
        globalAuthPda,
        globalState,
        gameVault,
        houseWallet,
        payer: userTokenAccount,
        userAccount: userSpinAccount,
        systemProgram: anchor.web3.SystemProgram.programId,
        feesWallet: houseWallet,
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
      tokenProgram: TOKEN_PROGRAM_ID,
    }).rpc(OPTS);
  });

});
