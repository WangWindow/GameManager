pub mod schema;

use std::path::Path;

pub async fn init_db(db_path: &Path) -> Result<toasty::Db, String> {
    if let Some(parent) = db_path.parent() {
        crate::util::path::ensure_dir(parent)?;
    }

    let db_exists = db_path.exists();
    let conn_str = format!("sqlite://{}", db_path.display());

    let db = toasty::Db::builder()
        .models(toasty::models!(
            crate::db::schema::Game,
            crate::db::schema::Engine,
            crate::db::schema::Setting,
        ))
        .connect(&conn_str)
        .await
        .map_err(|e| format!("数据库连接失败: {}", e))?;

    if !db_exists {
        db.push_schema()
            .await
            .map_err(|e| format!("数据库迁移失败: {}", e))?;
    }

    Ok(db)
}

pub async fn get_setting(db: &mut toasty::Db, key: &str) -> Result<Option<String>, String> {
    use schema::Setting;

    let setting = Setting::filter_by_key(key)
        .first()
        .exec(db)
        .await
        .map_err(|e| format!("查询设置失败: {}", e))?;
    Ok(setting.map(|s| s.value))
}

pub async fn set_setting(
    db: &mut toasty::Db,
    key: &str,
    value: &str,
) -> Result<(), String> {
    use schema::Setting;

    let existing = Setting::filter_by_key(key)
        .first()
        .exec(db)
        .await
        .map_err(|e| format!("查询设置失败: {}", e))?;

    if let Some(mut s) = existing {
        s.update()
            .value(value.to_string())
            .exec(db)
            .await
            .map_err(|e| format!("更新设置失败: {}", e))?;
    } else {
        toasty::create!(Setting {
            key: key.to_string(),
            value: value.to_string(),
        })
        .exec(db)
        .await
        .map_err(|e| format!("创建设置失败: {}", e))?;
    }

    Ok(())
}
