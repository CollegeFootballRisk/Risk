/// Merges the `source_user` into the `destination_user`
/// 
/// # Fields:
/// - `destination_user` (InternalUser): Represents the user to be merged into
/// - `source_user` (InternalUser): Represents the user to be merged
/// - `admin`(Option<Uuid>): If present, represents the Admin user authorizing the merge  
pub fn merge_user(destination_user: InternalUser, source_user: InternalUser, admin: Option<Uuid>) {
    // Cases:
    // 1. Neither user has any moves
    // 2. One user has moves, the other does not
    // 3. Both users have moves, but none overlap
    // 4. Both users have moves, but some overlap => requires admin to auth
    todo!()
}