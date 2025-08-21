pub fn get_random_u128(max: u128) -> anyhow::Result<u128, getrandom::Error>
{
        let mut buf = [0u8; 16];

        getrandom::fill(&mut buf)?;

        Ok(u128::from_ne_bytes(buf) % max)
}
