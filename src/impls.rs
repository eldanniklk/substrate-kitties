use super::*; 
use frame::prelude::*; 
use frame::primitives::BlakeTwo256; 
use frame::traits::tokens::Preservation; 
use frame::traits::Hash; 


impl<T: Config> Pallet<T> {

    // -------------------------------------------------------------------------
    //  Función: gen_dna()
    // -------------------------------------------------------------------------
    // Genera y devuelve un ADN único de 32 bytes para un nuevo kitty.
	// Se usa información del bloque actual y el contador de kitties para garantizar unicidad.
    pub fn gen_dna() -> [u8; 32] {
        // Crea una "semilla" única combinando varios valores del sistema.
        // Esto evita que dos kitties generados en el mismo bloque tengan el mismo ADN.
        let unique_payload = (
            frame_system::Pallet::<T>::parent_hash(),    // Hash del bloque anterior
            frame_system::Pallet::<T>::block_number(),   // Número del bloque actual
            frame_system::Pallet::<T>::extrinsic_index(),// Índice de la transacción dentro del bloque
            CountForKitties::<T>::get(),                 // Cantidad actual de kitties creados
        );

        // Aplica el hash Blake2-256 sobre el payload y convierte el resultado en [u8; 32].
        BlakeTwo256::hash_of(&unique_payload).into()
    }

    // -------------------------------------------------------------------------
    //  Función: mint()
    // -------------------------------------------------------------------------
    /// Crea un nuevo kitty y lo asigna al propietario indicado.
    /// Lanza errores si ya existe un kitty con ese ADN o si el propietario tiene demasiados.
    pub fn mint(owner: T::AccountId, dna: [u8; 32]) -> DispatchResult {
        // Crea la estructura del kitty con su ADN y dueño.
        let kitty = Kitty { dna, owner: owner.clone(), price: None };

        // Asegura que no exista otro kitty con el mismo ADN.
        ensure!(!Kitties::<T>::contains_key(dna), Error::<T>::DuplicateKitty);

        // Incrementa el contador global de kitties, validando overflow.
        let current_count: u32 = CountForKitties::<T>::get();
        let new_count = current_count.checked_add(1).ok_or(Error::<T>::TooManyKitties)?;

        // Añade el nuevo kitty al vector de kitties del propietario.
        KittiesOwned::<T>::try_append(&owner, dna).map_err(|_| Error::<T>::TooManyOwned)?;

        // Inserta el kitty en el mapa global de kitties.
        Kitties::<T>::insert(dna, kitty);

        // Actualiza el contador total de kitties.
        CountForKitties::<T>::set(new_count);

        // Emite un evento indicando la creación.
        Self::deposit_event(Event::<T>::Created { owner });

        Ok(())
    }

    // -------------------------------------------------------------------------
    //  Función: do_transfer()
    // -------------------------------------------------------------------------
    /// Transfiere un kitty de un usuario a otro, verificando propiedad, límites y validez.
    pub fn do_transfer(from: T::AccountId, to: T::AccountId, kitty_id: [u8; 32]) -> DispatchResult {
        // No se puede transferir un kitty a uno mismo.
        ensure!(from != to, Error::<T>::TransferToSelf);

        // Obtiene el kitty de almacenamiento, si no existe lanza error.
        let mut kitty = Kitties::<T>::get(kitty_id).ok_or(Error::<T>::NoKitty)?;

        // Verifica que quien realiza la operación sea el dueño actual.
        ensure!(kitty.owner == from, Error::<T>::NotOwner);

        // Actualiza el dueño y elimina el precio (ya no está en venta).
        kitty.owner = to.clone();
        kitty.price = None;

        // Obtiene el listado de kitties del receptor y añade el nuevo.
        let mut to_owned = KittiesOwned::<T>::get(&to);
        to_owned.try_push(kitty_id).map_err(|_| Error::<T>::TooManyOwned)?;

        // Elimina el kitty del listado del remitente.
        let mut from_owned = KittiesOwned::<T>::get(&from);
        if let Some(ind) = from_owned.iter().position(|&id| id == kitty_id) {
            from_owned.swap_remove(ind); // Remueve el elemento rápidamente (sin mantener orden).
        } else {
            return Err(Error::<T>::NoKitty.into());
        }

        // Actualiza almacenamiento: nuevo dueño y estado del kitty.
        Kitties::<T>::insert(kitty_id, kitty);
        KittiesOwned::<T>::insert(&to, to_owned);
        KittiesOwned::<T>::insert(&from, from_owned);

        // Emite evento de transferencia.
        Self::deposit_event(Event::<T>::Transferred { from, to, kitty_id });

        Ok(())
    }

    // -------------------------------------------------------------------------
    //  Función: do_set_price()
    // -------------------------------------------------------------------------
    // Permite al dueño establecer o quitar un precio de venta para su kitty.
    pub fn do_set_price(
        caller: T::AccountId,             // Quien realiza la llamada
        kitty_id: [u8; 32],               // ID del kitty
        new_price: Option<BalanceOf<T>>,  // Precio opcional (None = no venta)
    ) -> DispatchResult {
        // Verifica que el kitty exista.
        let mut kitty = Kitties::<T>::get(kitty_id).ok_or(Error::<T>::NoKitty)?;

        // Solo el dueño puede establecer el precio.
        ensure!(kitty.owner == caller, Error::<T>::NotOwner);

        // Actualiza el precio en la estructura.
        kitty.price = new_price;

        // Guarda los cambios en almacenamiento.
        Kitties::<T>::insert(kitty_id, kitty);

        // Emite evento de cambio de precio.
        Self::deposit_event(Event::<T>::PriceSet { owner: caller, kitty_id, new_price });

        Ok(())
    }

    // -------------------------------------------------------------------------
    //  Función: do_buy_kitty()
    // -------------------------------------------------------------------------
    // Permite a un comprador adquirir un kitty en venta si paga el precio correcto.
    pub fn do_buy_kitty(
        buyer: T::AccountId,        // Comprador
        kitty_id: [u8; 32],         // ID del kitty a comprar
        price: BalanceOf<T>,        // Precio máximo dispuesto a pagar
    ) -> DispatchResult {
        // Obtiene el kitty desde almacenamiento.
        let kitty = Kitties::<T>::get(kitty_id).ok_or(Error::<T>::NoKitty)?;

        // Verifica que esté en venta.
        let real_price = kitty.price.ok_or(Error::<T>::NotForSale)?;

        // Asegura que el comprador ofrece al menos el precio mínimo.
        ensure!(price >= real_price, Error::<T>::MaxPriceTooLow);

        // Transfiere los fondos al vendedor manteniendo el saldo vivo.
        T::NativeBalance::transfer(&buyer, &kitty.owner, real_price, Preservation::Preserve)?;

        // Transfiere la propiedad del kitty.
        Self::do_transfer(kitty.owner, buyer.clone(), kitty_id)?;

        // Emite evento de venta completada.
        Self::deposit_event(Event::<T>::Sold { buyer, kitty_id, price: real_price });

        Ok(())
    }
}
