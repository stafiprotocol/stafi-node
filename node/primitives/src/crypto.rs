// Copyright 2019-2020 Stafi Protocol.
// This file is part of Stafi.

// Stafi is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Stafi.  If not, see <http://www.gnu.org/licenses/>.

use super::{Signature, Verify};
use frame_system::offchain::AppCrypto;
use sp_core::crypto::{key_types, KeyTypeId};

/// Key type
pub const KEY_TYPE: KeyTypeId = key_types::AURA;

/// sr25519_app
pub mod sr25519_app {
    use sp_application_crypto::{app_crypto, sr25519};
    app_crypto!(sr25519, super::KEY_TYPE);
}

/// sr25519 AppCrypto
pub struct Sr25519AppCrypto;

impl AppCrypto<<Signature as Verify>::Signer, Signature> for Sr25519AppCrypto {
    type RuntimeAppPublic = sr25519_app::Public;
    type GenericSignature = sp_core::sr25519::Signature;
    type GenericPublic = sp_core::sr25519::Public;
}

/// ed25519_app
pub mod ed25519_app {
    use sp_application_crypto::{app_crypto, ed25519};
    app_crypto!(ed25519, super::KEY_TYPE);
}

/// ed25519 AppCrypto
pub struct Ed25519AppCrypto;

impl AppCrypto<<Signature as Verify>::Signer, Signature> for Ed25519AppCrypto {
    type RuntimeAppPublic = ed25519_app::Public;
    type GenericSignature = sp_core::ed25519::Signature;
    type GenericPublic = sp_core::ed25519::Public;
}

/// ecdsa_app
pub mod ecdsa_app {
    use sp_application_crypto::{app_crypto, ecdsa};
    app_crypto!(ecdsa, super::KEY_TYPE);
}

/// ecdsa AppCrypto
pub struct EcdsaAppCrypto;

impl AppCrypto<<Signature as Verify>::Signer, Signature> for EcdsaAppCrypto {
    type RuntimeAppPublic = ecdsa_app::Public;
    type GenericSignature = sp_core::ecdsa::Signature;
    type GenericPublic = sp_core::ecdsa::Public;
}