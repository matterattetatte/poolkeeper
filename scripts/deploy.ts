// scripts/deploy.ts
import { AnchorProvider, Program, web3 } from "@coral-xyz/anchor";
import { PublicKey } from "@solana/web3.js";
import { execSync } from "child_process";
import fs from "fs";
import path from "path";

async function main() {
  console.log("Building program...");
  execSync("anchor build", { stdio: "inherit" });

  const programId = getProgramIdFromTarget();
  console.log(`Program ID: ${programId.toBase58()}`);

  console.log("Deploying to devnet...");
  execSync(`anchor deploy --provider.cluster devnet`, { stdio: "inherit" });

  console.log("Verifying deployment...");
  const provider = AnchorProvider.env();
  const program = new Program(
    JSON.parse(fs.readFileSync("./target/idl/vault_manager.json", "utf8")),
    programId,
    provider
  );

  try {
    const state = await program.account.vault.fetchNullable(
      await getVaultPda(programId)
    );
    if (state) {
      console.log("Warning: Vault already exists at PDA");
    }
  } catch (err) {
    console.log("No existing vault found.");
  }

  console.log("Deployment complete!");
  console.log(`Program ID: ${programId.toBase58()}`);
  console.log(`IDL: target/idl/vault_manager.json`);
  console.log(`Run: anchor test --skip-local-validator`);
}

function getProgramIdFromTarget(): PublicKey {
  const idlPath = path.join(__dirname, "..", "target", "idl", "vault_manager.json");
  if (!fs.existsSync(idlPath)) {
    throw new Error("IDL not found. Run `anchor build` first.");
  }
  const idl = JSON.parse(fs.readFileSync(idlPath, "utf8"));
  return new PublicKey(idl.address);
}

async function getVaultPda(programId: PublicKey): Promise<PublicKey> {
  const [vaultPda] = PublicKey.findProgramAddressSync(
    [Buffer.from("vault"), new PublicKey("WHIRLPOOL_PUBKEY_HERE").toBuffer()],
    programId
  );
  return vaultPda;
}

main().catch((err) => {
  console.error("Deployment failed:", err);
  process.exit(1);
});