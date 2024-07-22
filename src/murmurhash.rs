use std::hash::{Hash, Hasher};

use fasthash::{murmur3, Murmur3HasherExt};

fn murmurhashV3<T: Hash>(t: &T) -> u64 {
    let mut s: Murmur3HasherExt = Default::default();
    t.hash(&mut s);
    s.finish()
}
