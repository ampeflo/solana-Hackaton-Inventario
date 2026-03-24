use anchor_lang::prelude::*;

declare_id!("44uEariW54B7XgxaUmQD4NPmnVuGSrqoGdixcQqznnZ8");

#[program]
pub mod inventario_program {
    use super::*;

    /// Inicializa una nueva cuenta de Inventario para el usuario.
    /// La cuenta es una PDA derivada del b"inventario" y la Pubkey del owner.
    pub fn crear_inventario(ctx: Context<CrearInventario>, nombre: String) -> Result<()> {
        let inventario = &mut ctx.accounts.inventario;
        inventario.nombre = nombre;
        inventario.owner = *ctx.accounts.owner.key;
        inventario.productos = Vec::new(); // Inicializamos el vector de productos vacío.
        msg!("¡Inventario '{}' creado exitosamente!", inventario.nombre);
        Ok(())
    }

    /// Agrega un nuevo producto al vector del inventario.
    /// Valida que no se supere el límite de 10 productos definido en la estructura.
    pub fn agregar_producto(
        ctx: Context<AgregarProducto>,
        nombre: String,
        especie: String,
        stock: u8,
    ) -> Result<()> {
        let inventario = &mut ctx.accounts.inventario;

        if inventario.productos.len() >= 10 {
            return err!(MyError::InventarioLleno);
        }

        let nuevo_producto = Producto {
            nombre,
            especie,
            stock,
        };

        // Imprimimos el mensaje ANTES de mover el producto al vector
        msg!("Producto '{}' agregado al inventario.", nuevo_producto.nombre);

        // Ahora sí lo movemos (aquí la variable nuevo_producto deja de ser válida)
        inventario.productos.push(nuevo_producto);
        
        Ok(())
    }

    /// Elimina un producto específico del vector buscando por su nombre exacto.
    pub fn eliminar_registro_producto(
        ctx: Context<EliminarRegistroProducto>,
        nombre_producto: String,
    ) -> Result<()> {
        let inventario = &mut ctx.accounts.inventario;

        // Buscamos la posición del producto.
        if let Some(pos) = inventario
            .productos
            .iter()
            .position(|producto| producto.nombre == nombre_producto)
        {
            // remove() reordena el vector, lo cual consume más gas, pero mantiene el orden.
            inventario.productos.remove(pos);
            msg!("Producto '{}' eliminado.", nombre_producto);
            Ok(())
        } else {
            // Error personalizado si el producto no existe.
            err!(MyError::ProductoNoEncontrado)
        }
    }

    /// Imprime en los logs del programa los productos actuales.
    /// Útil para depuración (Read operation).
    pub fn ver_productos(ctx: Context<VerInventario>) -> Result<()> {
        let inventario = &ctx.accounts.inventario;

        if inventario.productos.is_empty() {
            msg!("El inventario '{}' está vacío.", inventario.nombre);
            return Ok(());
        }

        msg!("--- Productos en '{}' ---", inventario.nombre);
        for (i, producto) in inventario.productos.iter().enumerate() {
            msg!(
                "#{}: {} ({}), Stock: {}",
                i + 1,
                producto.nombre,
                producto.especie,
                producto.stock
            );
        }
        Ok(())
    }

    /// Actualiza el stock de un producto existente buscando por su nombre.
    pub fn modificar_stock(
        ctx: Context<ModificarStock>,
        nombre_producto: String,
        nuevo_stock: u8,
    ) -> Result<()> {
        let inventario = &mut ctx.accounts.inventario;

        // Buscamos una referencia mutable al producto.
        let producto = inventario
            .productos
            .iter_mut()
            .find(|p| p.nombre == nombre_producto);

        match producto {
            Some(p) => {
                p.stock = nuevo_stock;
                msg!(
                    "Stock de '{}' actualizado a: {}",
                    nombre_producto,
                    nuevo_stock
                );
                Ok(())
            }
            None => err!(MyError::ProductoNoEncontrado),
        }
    }
}

// --- Estructuras de Datos (Cuentas) ---

#[account]
#[derive(InitSpace)] // Calcula automáticamente el espacio necesario en bytes.
pub struct Inventario {
    #[max_len(100)] // Límite para el String de nombre.
    pub nombre: String,
    pub owner: Pubkey, // Llave pública del dueño para validación.
    #[max_len(10)] // Límite estricto de 10 productos en el vector.
    pub productos: Vec<Producto>,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, InitSpace)]
pub struct Producto {
    #[max_len(60)]
    pub nombre: String,
    #[max_len(60)]
    pub especie: String,
    pub stock: u8, // u8 soporta de 0 a 255.
}

// --- Contextos de Validación de Cuentas ---

#[derive(Accounts)]
pub struct CrearInventario<'info> {
    // init: Crea la cuenta. payer: Quién paga la renta. space: Tamaño. seeds: Para derivar la PDA.
    #[account(
        init, 
        payer = owner, 
        space = 8 + Inventario::INIT_SPACE, // 8 bytes para el discriminador de Anchor.
        seeds = [b"inventario", owner.key().as_ref()], 
        bump
    )]
    pub inventario: Account<'info, Inventario>,
    #[account(mut)] // mut: La cuenta del pagador cambiará (su balance de SOL).
    pub owner: Signer<'info>, // Debe firmar la transacción.
    pub system_program: Program<'info, System>, // Requerido para crear cuentas.
}

#[derive(Accounts)]
pub struct AgregarProducto<'info> {
    // has_one = owner: Validación crítica de seguridad. Verifica que inventario.owner == owner.key().
    #[account(mut, has_one = owner)]
    pub inventario: Account<'info, Inventario>,
    pub owner: Signer<'info>, // Solo el dueño puede firmar para agregar.
}

#[derive(Accounts)]
pub struct EliminarRegistroProducto<'info> {
    #[account(mut, has_one = owner)]
    pub inventario: Account<'info, Inventario>,
    pub owner: Signer<'info>,
}

#[derive(Accounts)]
pub struct VerInventario<'info> {
    // Esta operación es de solo lectura, no requiere 'mut' ni 'Signer'.
    pub inventario: Account<'info, Inventario>,
}

#[derive(Accounts)]
pub struct ModificarStock<'info> {
    #[account(mut, has_one = owner)]
    pub inventario: Account<'info, Inventario>,
    pub owner: Signer<'info>,
}

// --- Errores Personalizados ---

#[error_code]
pub enum MyError {
    #[msg("El producto indicado no existe en este inventario.")]
    ProductoNoEncontrado,
    #[msg("El inventario ha alcanzado su capacidad máxima (10 productos).")]
    InventarioLleno,
}
