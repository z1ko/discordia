
use crate::affinity::{AffinityScore};

//
// Rappresenta utente conosciuto
//

#[derive(Debug, Clone)]
pub struct User 
{
    pub affinity: AffinityScore,
}