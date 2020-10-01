
pub mod misc;
pub mod music;
pub mod state;

// Risultato di un comando
#[derive(PartialEq)]
pub enum CmdResult
{
    Success(i32),   // Aggiunge exo specificata
    Skip,           // Non aumenta l'exp dell'utente
    Failure,
}