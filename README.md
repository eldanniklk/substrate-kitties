# SubstrateKittiesPallet

A pallet developed in **Substrate FRAME**, designed to manage in a decentralized way the creation, transfer, and buying/selling of *digital kitties*.  
The goal of **SubstrateKittiesPallet** is to demonstrate full mastery of Rust-based development on Substrate: structured storage, events, validations, and also advanced testing such as **fuzzing tests** to verify non-trivial behaviors.

---

## Pallet Structure

The pallet is divided into well-separated modules to maintain clarity and scalability:

- **`lib.rs`** → Main logic of the module.  
  Defines the *extrinsics* (`create_kitty`, `transfer`, `set_price`, `buy_kitty`) and the pallet’s core structure (storage, events, errors, configuration, etc.).

- **`impls.rs`** → Implementation of the pallet’s internal functions.  
  Here internal methods are defined, such as:
  - `gen_dna()` → Generates a unique DNA for each kitty using `BlakeTwo256`.
  - `mint()` → Creates a new kitty and registers it on the blockchain.
  - `do_transfer()` → Transfers kitty ownership between accounts.
  - `do_set_price()` → Assigns or updates a kitty’s price.
  - `do_buy_kitty()` → Allows purchasing a kitty if price and sale conditions are met.

- **`tests.rs`** → Includes classical unit tests and also **fuzzing tests** using `proptest`.

---

## 🧩 Key Features

- **Unique Kitty Creation:**  
  Each kitty has a unique DNA generated from block information (parent hash, block number, extrinsic index, global counter, etc.), ensuring uniqueness.

- **Secure Transfers:**  
  Ownership is validated before allowing a transfer. Transferring a kitty to oneself or to an unauthorized user is prevented.

- **Price and Marketplace Control:**  
  Users can list their kitties for sale and others can buy them, always ensuring price validity through native balance functions (`Mutate` and `Inspect`).

- **Efficient Storage Management:**  
  Each account has a maximum number of kitties (defined with `BoundedVec`), avoiding data saturation or storage abuse.

---

## Testing and Validation

The project includes two types of tests:

### Unit Tests
Verify expected behaviors under controlled conditions:
- Creating a kitty and storing it correctly.  
- Valid transfers between users.  
- Restrictions on ownership, duplication, and storage limits.  
These tests ensure the *functional correctness* of the code.

### Fuzzing Tests (with `proptest`)
**Fuzzing testing** generates **random and non-deterministic inputs** (e.g., variable-length DNA, non-existent kitty IDs, invalid prices, etc.) to try breaking the code and detect scenarios that unit tests do not cover.

Unlike unit tests (where the developer defines the test cases), fuzzing discovers cases automatically through random inputs, helping identify unexpected errors and ensuring higher robustness of the pallet.

When a fuzzing test fails, `proptest` shows a console message similar to:

thread 'tests::fuzzing_example' panicked at 'Test failed (seed: 432157)', ...

The process to handle it is as follows:  
1. Copy the **seed** or **specific failing input** shown in the console.  
2. Repeat the test with that fixed input in a new unit test to isolate the error.  
3. Fix the code or add an additional validation (`ensure!()`, limits, etc.).  
4. Run `cargo test` again until all tests pass successfully.

Because fuzzing generates different inputs each run, the same failure may not recur in future runs. Therefore, recording and debugging each failure is essential to maintain code integrity.

---

## Security and Best Practices

Although this project does not require key management or external APIs, in future iterations or real deployments it is recommended to:

- Use `.env` files to store sensitive keys or configurations.  
- Review the code before compiling or publishing.  
- Control storage limits (`BoundedVec`) to avoid data saturation.

---

## Future Improvements

- **Benchmarks:** Add performance tests to measure the *weight* of each operation, allowing optimization of resource usage and evaluation of the execution cost of each extrinsic on the blockchain.  
- **Cross-Chain Messaging (XCM):** Integrate communication between parachains using XCM to allow cross-chain transfers and operations of Kitties.  
- **Integration into a Full Runtime:** Extend the pallet to be part of a more complex Substrate runtime, enabling full compatibility with a functional blockchain and other standard pallets.  

---

## License
  
Created by **Daniel Sánchez** as part of advanced learning and mastery of **Substrate FRAME** and **Rust**.

---

> “Code quality is not only measured by whether it works,  
> but by whether it withstands everything the user did not expect it to do.”
