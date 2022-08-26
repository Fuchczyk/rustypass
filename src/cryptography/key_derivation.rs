use digest::Digest;

pub struct HashMachine<T>
where
    T: Digest,
{
    hash_struct: T,
}
