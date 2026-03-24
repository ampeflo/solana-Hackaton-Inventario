import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";

describe("inventario-test", () => {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);
  const program = anchor.workspace.InventarioProgram;

  it("¡Inicializa y Agrega Producto!", async () => {
    // Calculamos la PDA
    const [inventarioPda] = await anchor.web3.PublicKey.findProgramAddress(
      [Buffer.from("inventario"), provider.wallet.publicKey.toBuffer()],
      program.programId
    );

    // 1. Crear Inventario
    const tx1 = await program.methods
      .crearInventario("Veterianaria de Angel")
      .accounts({
        inventario: inventarioPda,
        owner: provider.wallet.publicKey,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .rpc();
    console.log("Inventario creado. Tx:", tx1);

    // 2. Agregar Producto
    const tx2 = await program.methods
      .agregarProducto("Papas", "Frituras", 10)
      .accounts({
        inventario: inventarioPda,
        owner: provider.wallet.publicKey,
      })
      .rpc();
    console.log("Producto agregado. Tx:", tx2);
  });
});
