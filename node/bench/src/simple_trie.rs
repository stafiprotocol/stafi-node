// Copyright 2019-2020 Stafi Protocol.
// This file is part of Stafi.

// Stafi is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Stafi.  If not, see <http://www.gnu.org/licenses/>.

use std::{collections::HashMap, sync::Arc};

use kvdb::KeyValueDB;
use node_primitives::Hash;
use sp_trie::DBValue;
use hash_db::{HashDB, AsHashDB, Prefix, Hasher as _};

pub type Hasher = sp_core::Blake2Hasher;

/// Immutable generated trie database with root.
pub struct SimpleTrie<'a> {
	pub db: Arc<dyn KeyValueDB>,
	pub overlay: &'a mut HashMap<Vec<u8>, Option<Vec<u8>>>,
}

impl<'a> AsHashDB<Hasher, DBValue> for SimpleTrie<'a> {
	fn as_hash_db(&self) -> &dyn hash_db::HashDB<Hasher, DBValue> { &*self }

	fn as_hash_db_mut<'b>(&'b mut self) -> &'b mut (dyn HashDB<Hasher, DBValue> + 'b) {
		&mut *self
	}
}

impl<'a> HashDB<Hasher, DBValue> for SimpleTrie<'a> {
	fn get(&self, key: &Hash, prefix: Prefix) -> Option<DBValue> {
		let key = sp_trie::prefixed_key::<Hasher>(key, prefix);
		if let Some(value) = self.overlay.get(&key) {
			return value.clone();
		}
		self.db.get(0, &key).expect("Database backend error")
	}

	fn contains(&self, hash: &Hash, prefix: Prefix) -> bool {
		self.get(hash, prefix).is_some()
	}

	fn insert(&mut self, prefix: Prefix, value: &[u8]) -> Hash {
		let key = Hasher::hash(value);
		self.emplace(key, prefix, value.to_vec());
		key
	}

	fn emplace(&mut self, key: Hash, prefix: Prefix, value: DBValue) {
		let key = sp_trie::prefixed_key::<Hasher>(&key, prefix);
		self.overlay.insert(key, Some(value));
	}

	fn remove(&mut self, key: &Hash, prefix: Prefix) {
		let key = sp_trie::prefixed_key::<Hasher>(key, prefix);
		self.overlay.insert(key, None);
	}
}
