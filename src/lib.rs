#![cfg_attr(not(feature = "std"), no_std)]


mod impls; 
mod tests; 

use frame::prelude::*;
use frame::traits::fungible::Inspect; 
use frame::traits::fungible::Mutate;  
pub use pallet::*; 

#[frame::pallet(dev_mode)]
pub mod pallet {
    use super::*; 

    // --- Declaración principal del pallet ---
    #[pallet::pallet]
    pub struct Pallet<T>(core::marker::PhantomData<T>);
    // Estructura principal del pallet.
    // `PhantomData` indica que este pallet depende del tipo genérico T (que implementa Config),
    // aunque no tengamos un campo real de ese tipo.

    // --- Configuración del pallet ---
    #[pallet::config]
    pub trait Config: frame_system::Config {
        // Tipo de evento que usará el runtime cuando esta paleta emita eventos.
        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

        /// Manejador de balance nativo (para operaciones de compra/venta).
        type NativeBalance: Inspect<Self::AccountId> + Mutate<Self::AccountId>;
    }

    // Alias para obtener fácilmente el tipo de balance del runtime.
    // Esto simplifica el código en vez de escribir toda la cadena de tipos cada vez.
    pub type BalanceOf<T> =
        <<T as Config>::NativeBalance as Inspect<<T as frame_system::Config>::AccountId>>::Balance;

    // --- Definición de la estructura Kitty ---
    #[derive(Encode, Decode, TypeInfo, MaxEncodedLen)]
    #[scale_info(skip_type_params(T))]
    pub struct Kitty<T: Config> {
        pub dna: [u8; 32],              // ADN del kitty (identificador único de 32 bytes)
        pub owner: T::AccountId,        // Dueño actual del kitty
        pub price: Option<BalanceOf<T>> // Precio actual (None si no está en venta)
    }

    // --- Almacenamientos del pallet ---
    #[pallet::storage]
    pub(super) type CountForKitties<T: Config> = StorageValue<Value = u32, QueryKind = ValueQuery>;
    // Guarda el número total de kitties creados.
    // QueryKind = ValueQuery indica que si no hay valor, devuelve 0 por defecto.

    #[pallet::storage]
    pub(super) type Kitties<T: Config> = StorageMap<Key = [u8; 32], Value = Kitty<T>>;
    // Mapa principal que guarda todos los kitties creados, usando su ADN (kitty_id) como clave.

    #[pallet::storage]
    pub(super) type KittiesOwned<T: Config> = StorageMap<
        Key = T::AccountId,
        Value = BoundedVec<[u8; 32], ConstU32<100>>,
        QueryKind = ValueQuery,
    >;
    // Mapa que almacena los IDs de los kitties propiedad de cada usuario.
    // Se limita a 100 kitties por usuario (BoundedVec) para evitar abusos o overflows.

    // --- Eventos del pallet ---
    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        Created { owner: T::AccountId }, // Emitido cuando se crea un nuevo kitty
        Transferred {                   // Emitido cuando se transfiere un kitty
            from: T::AccountId,
            to: T::AccountId,
            kitty_id: [u8; 32]
        },
        PriceSet {                      // Emitido cuando un dueño pone o quita un precio
            owner: T::AccountId,
            kitty_id: [u8; 32],
            new_price: Option<BalanceOf<T>>
        },
        Sold {                          // Emitido cuando se vende un kitty
            buyer: T::AccountId,
            kitty_id: [u8; 32],
            price: BalanceOf<T>
        },
    }

    // --- Errores posibles del pallet ---
    #[pallet::error]
    pub enum Error<T> {
        TooManyKitties,   // Se excedió el límite total de kitties permitidos
        DuplicateKitty,   // Ya existe un kitty con ese ADN
        TooManyOwned,     // El dueño ya posee el máximo de 100 kitties
        TransferToSelf,   // No se puede transferir un kitty a uno mismo
        NoKitty,          // El kitty no existe en el mapa
        NotOwner,         // La cuenta que intenta operar no es el dueño del kitty
        NotForSale,       // Se intenta comprar un kitty que no está en venta
        MaxPriceTooLow,   // El precio máximo ofrecido por el comprador es menor al precio de venta
    }

    // --- Extrinsics (funciones públicas que pueden llamarse desde fuera del runtime) ---
    #[pallet::call]
    impl<T: Config> Pallet<T> {

        /// Crea un nuevo kitty con ADN aleatorio y lo asigna al usuario que ejecuta la transacción.
        pub fn create_kitty(origin: OriginFor<T>) -> DispatchResult {
            let who = ensure_signed(origin)?; // Comprueba que la llamada proviene de una cuenta firmada (no root).
            let dna = Self::gen_dna(); // Genera un ADN aleatorio.
            Self::mint(who, dna)?; // Crea el kitty y lo asigna al dueño llamando a la función mint() (implementada en impls.rs)
            Ok(())
        }

        /// Transfiere un kitty a otra cuenta.
        pub fn transfer(
            origin: OriginFor<T>,
            to: T::AccountId,
            kitty_id: [u8; 32],
        ) -> DispatchResult {
            let who = ensure_signed(origin)?; // Verifica que la transacción esté firmada.
            Self::do_transfer(who, to, kitty_id)?; // Ejecuta la lógica de transferencia (valida, actualiza almacenamiento, emite evento).
            Ok(())
        }

        /// Permite poner un kitty en venta o quitarlo (establecer precio o None).
        pub fn set_price(
            origin: OriginFor<T>,
            kitty_id: [u8; 32],
            new_price: Option<BalanceOf<T>>,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?; // Comprueba que sea una cuenta firmada.
            Self::do_set_price(who, kitty_id, new_price)?; // Llama a la lógica de negocio para actualizar el precio.
            Ok(())
        }

        /// Permite comprar un kitty si está en venta y el comprador ofrece suficiente balance.
        pub fn buy_kitty(
            origin: OriginFor<T>,
            kitty_id: [u8; 32],
            max_price: BalanceOf<T>,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?; // Verifica que el comprador sea una cuenta válida.
            Self::do_buy_kitty(who, kitty_id, max_price)?; // Ejecuta la lógica de compra (valida precio, transfiere fondos, cambia dueño).
            Ok(())
        }
    }
}
