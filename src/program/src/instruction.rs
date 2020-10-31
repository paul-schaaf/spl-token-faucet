pub enum EscrowInstruction {
    /// Starts the trade by creating and populating an escrow account and transferring ownership of the given temp token account to the DPA
    /// 
    /// 
    /// Accounts expected:
    /// 
    /// 0. `[signer]` The account of the person initializing the escrow
    /// 1. `[writable]` Temporary token account that should be created prior to this instruction and owned by the initializer
    /// 2. `[]` The initializer's token account for the token they will receive should the trade go through
    /// 3. `[writable]` The escrow account, it will hold all necessary info about the trade.
    /// This acc SHOULD be created atomically in the same tx as calling `InitEscrow`. Otherwise another party can acquire ownership of the uninitialized account.
    /// 4. `[optional]` The account of the only person the initializer accepts for the deal, if left empty, anyone can take the other side of the escrow
    InitEscrow {
        /// the amount the initializer expects to be paid in the other token, as a u64 because that's the max possible supply of a token
        amount: u64
    },
    /// Accepts a trade
    /// 
    /// 
    /// Accounts expected:
    /// 
    /// 0. `[signer]` The account of the person taking the trade
    /// 1. `[writable]` Tempory token account of the token that will be sent by the taker
    /// 2. `[]` The taker's token account for the token they will receive should the trade go through
    /// 3. `[writable]` The escrow account created by the initializer to hold the escrow info
    Exchange {
        /// the amount the taker expects to be paid in the other token, as a u64 because that's the max possible supply of a token
        amount: u64
    },
    /// TODO
    Cancel
}
