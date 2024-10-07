use serde::Serialize;


#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Jogador {
    nome: String,
    apelido: String,
}

pub fn get_list() -> Vec<Jogador> {
    vec![
        Jogador {
            nome: "Ronaldo".to_string(),
            apelido: "Fenômeno".to_string(),
        },
        Jogador {
            nome: "Cristiano".to_string(),
            apelido: "CR7".to_string(),
        },
        Jogador {
            nome: "Vini".to_string(),
            apelido: "La Pulga".to_string(),
        },
    ]
}

pub fn get_max_jogadores() -> usize {
    3
}