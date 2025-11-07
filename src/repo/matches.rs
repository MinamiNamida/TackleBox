use crate::repo::error::RepoError;
use sqlx::{query, query_as, query_scalar, PgPool, Postgres, Transaction};
use std::sync::Arc;
use tackle_box::contracts::payloads::{GetMatchResponse, GetOnlineMatchResponse, MatchStatus};
use uuid::Uuid;

pub struct NewMatchDTO {
    pub name: String,
    pub game_type_id: Uuid,
    pub total_games: i32,
    pub creater_id: Uuid,
    pub password: Option<String>,
}

// #[derive(FromRow, Serialize)]
// pub struct GetMatchDTO {
//     pub match_id: Uuid,
//     pub match_name: String,
//     pub creater_id: Uuid,
//     pub creater_name: String,
//     pub winner_id: Option<Uuid>,
//     pub winner_agent_name: Option<String>,
//     pub game_type_id: Uuid,
//     pub game_type_name: String,
//     pub password: Option<String>,
//     pub total_games: i32,
//     pub status: MatchStatus,
//     pub start_time: DateTime<Utc>,
//     pub end_time: Option<DateTime<Utc>>,
// }

// #[derive(FromRow, Serialize)]
// pub struct GetOnlineMatchDTO {
//     pub match_id: Uuid,
//     pub match_name: String,
//     pub creater_id: Uuid,
//     pub creater_name: String,
//     pub game_type_id: Uuid,
//     pub game_type_name: String,
//     pub total_games: i32,
//     pub max_slots: i32,
//     pub min_slots: i32,
//     pub status: MatchStatus,
//     pub start_time: DateTime<Utc>,
//     pub with_password: bool,
// }

pub struct MatchRepo {
    pub pool: Arc<PgPool>,
}

impl MatchRepo {
    pub async fn get_transaction(&self) -> Result<Transaction<'_, Postgres>, RepoError> {
        Ok(self.pool.begin().await?)
    }

    pub async fn new_match(&self, one_match: NewMatchDTO) -> Result<Uuid, RepoError> {
        let mut conn = self.pool.acquire().await?;
        let match_id = query_scalar!(
            r#"
            insert into matches (name, game_type_id, total_games, creater_id, status, password)
            values ($1, $2, $3, $4, $5::match_status, $6) returning match_id;
            "#,
            one_match.name,
            one_match.game_type_id,
            one_match.total_games,
            one_match.creater_id,
            MatchStatus::Pending as MatchStatus,
            one_match.password,
        )
        .fetch_one(&mut *conn)
        .await?;
        Ok(match_id)
    }

    // pub async fn get_match_id_by_name(
    //     &self,
    //     user_id: Uuid,
    //     match_name: &String,
    // ) -> Result<Uuid, RepoError> {
    //     let mut conn = self.pool.acquire().await?;
    //     let match_id = query_scalar!(
    //         "select id from matches where creater_id = $1 and name = $2",
    //         user_id,
    //         match_name
    //     )
    //     .fetch_one(&mut *conn)
    //     .await?;
    //     Ok(match_id)
    // }

    pub async fn get_match(&self, match_id: Uuid) -> Result<GetMatchResponse, RepoError> {
        let mut conn = self.pool.acquire().await?;
        let one_match = query_as!(
            GetMatchResponse,
            r#"
            SELECT 
                M.match_id,
                M.name as match_name,
                M.game_type_id,
                G.name as game_type_name,
                M.creater_id,
                U.username AS creater_name,
                M.total_games,
                M.winner_id,
                WA.name AS "winner_agent_name: _",
                M.start_time,
                M.end_time,
                M.status as "status!:MatchStatus",
                M.password,
                G.min_slots,
                G.max_slots
            FROM 
                matches AS M
            INNER JOIN 
                gametypes AS G ON M.game_type_id = G.game_type_id
            INNER JOIN
                users AS U ON M.creater_id = U.user_id
            LEFT JOIN
                agents AS WA ON M.winner_id = WA.agent_id
            WHERE 
                M.match_id = $1
            "#,
            match_id
        )
        .fetch_one(&mut *conn)
        .await?;
        Ok(one_match)
    }

    // pub async fn get_match(&self, match_name: &String) -> Result<GetMatchDTO, RepoError> {
    //     let mut conn = self.pool.acquire().await?;
    //     let one_match = query_as!(
    //         GetMatchDTO,
    //         r#"
    //         select match_id as "match_id!",
    //         readable_match_name as "readable_match_name!",
    //         match_name_base as "match_name_base!",
    //         creater_username as "creater_username!",
    //         game_type as "game_type!",
    //         total_games as "total_games!",
    //         creater_id as "creater_id!",
    //         winner_id,
    //         winner_agent_readable_name,
    //         status as "status!:MatchStatus",
    //         start_time as "start_time!",
    //         end_time,
    //         password
    //         from v_readable_matches
    //         where readable_match_name = $1
    //         "#,
    //         match_name
    //     )
    //     .fetch_one(&mut *conn)
    //     .await?;
    //     Ok(one_match)
    // }

    pub async fn get_my_matches(&self, user_id: Uuid) -> Result<Vec<GetMatchResponse>, RepoError> {
        let mut conn = self.pool.acquire().await?;
        let matches = query_as!(
            GetMatchResponse,
            r#"
            SELECT 
                M.match_id,
                M.name as match_name,
                M.game_type_id,
                G.name as game_type_name,
                M.creater_id,
                U.username AS creater_name,
                M.total_games,
                M.winner_id,
                WA.name AS "winner_agent_name: _",
                M.start_time,
                M.end_time,
                M.status as "status!:MatchStatus",
                M.password,
                G.min_slots,
                G.max_slots
            FROM 
                matches AS M
            INNER JOIN 
                gametypes AS G ON M.game_type_id = G.game_type_id
            INNER JOIN
                users AS U ON M.creater_id = U.user_id
            LEFT JOIN
                agents AS WA ON M.winner_id = WA.agent_id
            WHERE
                M.creater_id = $1
            "#,
            user_id
        )
        .fetch_all(&mut *conn)
        .await?;
        Ok(matches)
    }

    pub async fn get_my_joined_matches(
        &self,
        user_id: Uuid,
    ) -> Result<Vec<GetMatchResponse>, RepoError> {
        let mut conn = self.pool.acquire().await?;

        let matches = query_as!(
            GetMatchResponse,
            r#"
            SELECT
                M.match_id,
                M.name AS match_name,
                M.game_type_id,
                G.name AS game_type_name,
                M.creater_id,
                CR.username AS creater_name, 
                M.total_games,
                M.winner_id,
                WA.name AS "winner_agent_name: _", 
                M.start_time,
                M.end_time,
                M.status AS "status!:MatchStatus", 
                M.password,
                G.min_slots,
                G.max_slots
            FROM 
                matches AS M
            INNER JOIN 
                gametypes AS G ON M.game_type_id = G.game_type_id
            INNER JOIN
                users AS CR ON M.creater_id = CR.user_id
            INNER JOIN 
                participants AS P ON M.match_id = P.match_id
            INNER JOIN
                agents AS A ON P.agent_id = A.agent_id
            LEFT JOIN
                agents AS WA ON M.winner_id = WA.agent_id
            WHERE
                A.owner_id = $1
            GROUP BY
                M.match_id, G.name, CR.username, WA.name, G.min_slots, G.max_slots -- 必须 Group By 以处理多 Agent 参与同一比赛的情况
            ORDER BY
                M.start_time DESC
            "#,
            user_id,
        ).fetch_all(&mut *conn)
        .await?;

        Ok(matches)
    }

    pub async fn get_online_matches(&self) -> Result<Vec<GetOnlineMatchResponse>, RepoError> {
        let mut conn = self.pool.acquire().await?;
        let matches = query_as!(
            GetOnlineMatchResponse,
            r#"
            SELECT 
                M.match_id,
                M.name AS match_name,
                M.creater_id,
                U.username AS creater_name,
                M.game_type_id,
                G.name AS game_type_name,
                M.total_games,
                M.start_time,
                M.status AS "status!:MatchStatus",
                M.password IS NOT NULL AS "with_password!",
                G.max_slots,
                G.min_slots,
                COUNT(P.match_id) AS "current_slots!" 
            FROM
                matches AS M
            INNER JOIN
                gametypes AS G ON M.game_type_id = G.game_type_id
            INNER JOIN
                users AS U ON M.creater_id = U.user_id
            LEFT JOIN
                participants AS P ON M.match_id = P.match_id
            WHERE
                M.status = $1
            GROUP BY
                M.match_id, U.username, G.name, G.max_slots, G.min_slots
            ORDER BY
                M.match_id;
            "#,
            MatchStatus::Pending as MatchStatus
        )
        .fetch_all(&mut *conn)
        .await?;
        Ok(matches)
    }

    pub async fn get_match_status(&self, match_id: Uuid) -> Result<MatchStatus, RepoError> {
        let mut conn = self.pool.acquire().await?;
        let status: MatchStatus = query_scalar!(
            r#"
            SELECT status as "status!: MatchStatus" FROM "matches" WHERE match_id = $1;
            "#,
            match_id
        )
        .fetch_one(&mut *conn)
        .await?;
        Ok(status)
    }

    pub async fn update_match_status(
        &self,
        match_id: Uuid,
        status: MatchStatus,
    ) -> Result<(), RepoError> {
        let mut conn = self.pool.acquire().await?;
        let _ = query!(
            r#"
            update matches 
            set status = $1
            where match_id = $2
            "#,
            status as MatchStatus,
            match_id,
        )
        .execute(&mut *conn)
        .await?;
        Ok(())
    }

    pub async fn delete_match(&self, match_id: Uuid, creater_id: Uuid) -> Result<(), RepoError> {
        let mut conn = self.pool.acquire().await?;
        let _ = query!(
            r#"
            delete from matches where match_id = $1 and creater_id = $2;
            "#,
            match_id,
            creater_id,
        )
        .execute(&mut *conn)
        .await?;
        Ok(())
    }

    pub async fn update_match_final_status(
        &self,
        tx: &mut Transaction<'_, Postgres>,
        match_id: Uuid,
        winner_id: Option<Uuid>,
    ) -> Result<(), RepoError> {
        let _ = query!(
            r#"
            update matches set status = $1, winner_id = $2 where match_id = $3
            "#,
            MatchStatus::Completed as MatchStatus,
            winner_id,
            match_id
        )
        .execute(tx.as_mut())
        .await?;
        Ok(())
    }
}
