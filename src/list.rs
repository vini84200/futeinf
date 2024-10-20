use crate::error::Result;
use serde::{Deserialize, Serialize};
use sqlx::{Pool, Postgres};

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Jogador {
    pub nome: String,
    pub id: i32,
    pub apelido: String,
}

#[derive(Deserialize, Debug)]
pub struct Ticket {
    status: String,
    id: i32,
    event_id: i32,
    full_name: Option<String>,
    email_address: Option<String>,
}

#[derive(Deserialize, Debug)]
pub struct TicketFieldValue {
    ticket_id_fk: Option<i32>,
    field_value: Option<String>,
    field_configuration_id_fk: Option<i64>,
}


pub async fn get_list(db: Pool<Postgres>) -> Result<Vec<Jogador>> {
    // TODO: encontrar no bd quais são os ids dos eventos e campo mais recente
    let event_id = 2;
    let campo_camisa = 2;
    //TODO: Use event_id from outra fonte
    let tickets = sqlx::query_as!(
        Ticket,
        "SELECT id, status, event_id, full_name, email_address FROM ticket WHERE STATUS='ACQUIRED' AND event_id=2"
    )
        .fetch_all(&db)
        .await;
    let tickets_fields = sqlx::query_as!(
        TicketFieldValue,
        "SELECT ticket_id_fk, field_value, field_configuration_id_fk FROM all_ticket_field_values 
            WHERE field_configuration_id_fk = $1
        ",
        campo_camisa
    )
        .fetch_all(&db)
        .await?;


    Ok(tickets_fields.iter()
        .map(|fv| {
            let camisa = fv.field_value.clone().unwrap();
            Jogador {
                nome: camisa.clone(),
                apelido: camisa,
                id: fv.ticket_id_fk.unwrap(),
            }
        })
        .collect()
    )
}

pub fn get_max_jogadores() -> usize {
    24
}

