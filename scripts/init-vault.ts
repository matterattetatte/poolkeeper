import {
    AnchorProvider,
    Program,
    web3,
    BN,
} from "@coral-xyz/anchor";
import {
    PublicKey,
    Keypair,
    SystemProgram,
    Transaction,
} from "@solana/web3.js";
import {
    TOKEN_PROGRAM_ID,
    ASSOCIATED_TOKEN_PROGRAM_ID,
    getAssociatedTokenAddressSync,
} from "@solana/spl-token";
import fs from "fs";
import path from "path";

// === CONFIG ===
const WHIRLPOOL_PUBKEY = new PublicKey(
    "H1esoY4v3aUcJ6bU8V3bS5yKxYv3V3bS5yKxYv3V3bS5" // Replace with real pool
);
// =============

async function main() {
    const provider = AnchorProvider.env();
    const connection = provider.connection;
    const wallet = provider.wallet;

    // Load IDL
    const idlPath = path.join(process.cwd(), "target", "idl", "vault_manager.json");
    const idl = JSON.parse(fs.readFileSync(idlPath, "utf8"));
    const program = new Program(idl, provider);

    const whirlpool = WHIRLPOOL_PUBKEY;

    // === Derive PDAs ===
    const [vaultPda, vaultBump] = PublicKey.findProgramAddressSync(
        [Buffer.from("vault"), whirlpool.toBuffer()],
        program.programId
    );

    const positionMint = Keypair.generate();

    // === Get Whirlpool state to read mints ===
    const whirlpoolAccount = await connection.getAccountInfo(whirlpool);
    if (!whirlpoolAccount) throw new Error("Whirlpool not found");

    // Parse mints from Whirlpool (first 64 bytes after discriminator)
    const data = whirlpoolAccount.data;
    const tokenMintA = new PublicKey(data.subarray(8, 40));
    const tokenMintB = new PublicKey(data.subarray(40, 72));

    console.log("Token A (Mint):", tokenMintA.toBase58());
    console.log("Token B (Mint):", tokenMintB.toBase58());

    // === Create vault token accounts (ATA style) ===
    const vaultTokenA = getAssociatedTokenAddressSync(tokenMintA, vaultPda, true);
    const vaultTokenB = getAssociatedTokenAddressSync(tokenMintB, vaultPda, true);

    // === Position PDA (Whirlpool owns it) ===
    const [positionPda] = PublicKey.findProgramAddressSync(
        [Buffer.from("position"), positionMint.publicKey.toBuffer()],
        new PublicKey("whirp111111111111111111111111111111111") // Whirlpool program ID
    );

    // === Position token account (vault owns the NFT) ===
    const positionTokenAccount = getAssociatedTokenAddressSync(
        positionMint.publicKey,
        vaultPda,
        true
    );

    // === Build transaction: create ATAs + initialize vault ===
    const tx = new Transaction();

    // Create ATA for vault_token_a
    if (!(await connection.getAccountInfo(vaultTokenA))) {
        tx.add(
            web3.AssociatedTokenProgram.createAssociatedTokenAccountInstruction(
                wallet.publicKey,
                vaultTokenA,
                vaultPda,
                tokenMintA
            )
        );
    }

    // Create ATA for vault_token_b
    if (!(await connection.getAccountInfo(vaultTokenB))) {
        tx.add(
            web3.AssociatedTokenProgram.createAssociatedTokenAccountInstruction(
                wallet.publicKey,
                vaultTokenB,
                vaultPda,
                tokenMintB
            )
        );
    }

    // Create ATA for position NFT
    if (!(await connection.getAccountInfo(positionTokenAccount))) {
        tx.add(
            web3.AssociatedTokenProgram.createAssociatedTokenAccountInstruction(
                wallet.publicKey,
                positionTokenAccount,
                vaultPda,
                positionMint.publicKey
            )
        );
    }

    // Initialize vault
    tx.add(
        await program.methods
            .initializeVault(vaultBump)
            .accounts({
                vault: vaultPda,
                whirlpool,
                tokenAMint: tokenMintA,
                tokenBMint: tokenMintB,
                vaultTokenA,
                vaultTokenB,
                position: positionPda,
                positionMint: positionMint.publicKey,
                positionTokenAccount,
                authority: wallet.publicKey,
                systemProgram: SystemProgram.programId,
                tokenProgram: TOKEN_PROGRAM_ID,
                rent: web3.SYSVAR_RENT_PUBKEY,
            })
            .instruction()
    );

    console.log("Sending transaction...");
    const sig = await provider.sendAndConfirm(tx, [positionMint]);
    console.log("Transaction confirmed:", `https://explorer.solana.com/tx/${sig}?cluster=devnet`);

    console.log("Vault initialized!");
    console.log("Vault PDA:", vaultPda.toBase58());
    console.log("Position Mint:", positionMint.publicKey.toBase58());
}

main().catch((err) => {
    console.error("Failed:", err);
    process.exit(1);
});