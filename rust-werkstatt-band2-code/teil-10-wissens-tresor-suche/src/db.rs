//! Teil 10: Der Wissens-Tresor – Datenbankanbindung & FTS5
//! Die Rust-Werkstatt – Band 2: Native Desktop-Apps mit Iced
//! Autor: Volker Dähn

use sqlx::sqlite::{SqliteConnectOptions, SqlitePool};
use std::str::FromStr;

#[derive(Debug, Clone, sqlx::FromRow)]
pub(crate) struct NotizEintrag {
    pub(crate) id: i64,
    pub(crate) titel: String,
    pub(crate) inhalt: String,
}

#[derive(Debug, Clone)]
pub(crate) struct TresorDb {
    pool: SqlitePool,
}

impl TresorDb {
    pub(crate) async fn oeffnen(
        pfad: &str,
    ) -> Result<Self, sqlx::Error> {
        let optionen = SqliteConnectOptions::from_str(pfad)?
            .create_if_missing(true);

        let pool = SqlitePool::connect_with(optionen).await?;

        // 1. Basis-Tabelle
        sqlx::query(
            "CREATE TABLE IF NOT EXISTS notizen (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                titel TEXT NOT NULL,
                inhalt TEXT NOT NULL
            );",
        )
        .execute(&pool)
        .await?;

        // 2. FTS5-Volltext-Tabelle
        sqlx::query(
            "CREATE VIRTUAL TABLE IF NOT EXISTS notizen_suche USING fts5(
                titel,
                inhalt,
                content='notizen',
                content_rowid='id'
            );",
        )
        .execute(&pool)
        .await?;

        // 3. Trigger für automatische FTS-Synchronisation
        sqlx::query(
            "CREATE TRIGGER IF NOT EXISTS notizen_ai AFTER INSERT ON notizen BEGIN
                INSERT INTO notizen_suche(rowid, titel, inhalt)
                VALUES (new.id, new.titel, new.inhalt);
            END;",
        )
        .execute(&pool)
        .await?;

        sqlx::query(
            "CREATE TRIGGER IF NOT EXISTS notizen_ad AFTER DELETE ON notizen BEGIN
                INSERT INTO notizen_suche(notizen_suche, rowid, titel, inhalt)
                VALUES('delete', old.id, old.titel, old.inhalt);
            END;",
        )
        .execute(&pool)
        .await?;

        sqlx::query(
            "CREATE TRIGGER IF NOT EXISTS notizen_au AFTER UPDATE ON notizen BEGIN
                INSERT INTO notizen_suche(notizen_suche, rowid, titel, inhalt)
                VALUES('delete', old.id, old.titel, old.inhalt);
                INSERT INTO notizen_suche(rowid, titel, inhalt)
                VALUES (new.id, new.titel, new.inhalt);
            END;",
        )
        .execute(&pool)
        .await?;

        Ok(Self { pool })
    }

    pub(crate) async fn alle_notizen(
        &self,
    ) -> Result<Vec<NotizEintrag>, sqlx::Error> {
        sqlx::query_as::<_, NotizEintrag>(
            "SELECT id, titel, inhalt FROM notizen ORDER BY id DESC;",
        )
        .fetch_all(&self.pool)
        .await
    }

    pub(crate) async fn fts_suchen(
        &self,
        suchtext: &str,
    ) -> Result<Vec<NotizEintrag>, sqlx::Error> {
        let muster = format!("{}*", suchtext.trim());
        sqlx::query_as::<_, NotizEintrag>(
            "SELECT id, titel, inhalt FROM notizen
             WHERE id IN (
                 SELECT rowid FROM notizen_suche
                 WHERE notizen_suche MATCH ?
             )
             ORDER BY id DESC;",
        )
        .bind(muster)
        .fetch_all(&self.pool)
        .await
    }

    pub(crate) async fn notiz_einfuegen(
        &self,
        titel: &str,
        inhalt: &str,
    ) -> Result<i64, sqlx::Error> {
        let res = sqlx::query(
            "INSERT INTO notizen (titel, inhalt) VALUES (?, ?);",
        )
        .bind(titel)
        .bind(inhalt)
        .execute(&self.pool)
        .await?;

        Ok(res.last_insert_rowid())
    }

    pub(crate) async fn notiz_aktualisieren(
        &self,
        id: i64,
        titel: &str,
        inhalt: &str,
    ) -> Result<(), sqlx::Error> {
        sqlx::query(
            "UPDATE notizen SET titel = ?, inhalt = ? WHERE id = ?;",
        )
        .bind(titel)
        .bind(inhalt)
        .bind(id)
        .execute(&self.pool)
        .await?;

        Ok(())
    }
}
