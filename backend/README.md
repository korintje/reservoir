# Reservoir backend

Rust / Actix Web / sqlx / SQLiteによる予約APIです。APIはすべて/api配下で公開します。Nginxは/apiを除去せず転送してください。

## 起動

```bash
cargo build --locked
cargo test --locked
cargo clippy --all-targets --locked -- -D warnings
```

通常の開発起動はルートのpython scripts/dev.pyを使用します。手動起動ではreservoir hash-passwordへ標準入力から管理者パスワードを渡し、得られたハッシュを設定してください。設定ファイルはRESERVOIR_CONFIGで指定でき、既定はreservoir.tomlです。

```toml
web_url = "127.0.0.1:8080"
db_path = "/var/lib/reservoir/reservoir.db"
admin_password_hash = "$argon2id$..."
secure_cookies = true
basic_auth_enabled = true
public_mode = "lan"
```

DBの親ディレクトリーは作成しておきます。起動時にマイグレーションと旧データの検査を行います。空の管理者ハッシュでは起動しません。basic_auth_enabled/public_modeは状態表示用であり、アクセス制御はNginxとOSファイアウォールで行います。

## API

| メソッドとパス（/apiの後） | 用途・認証 |
| --- | --- |
| GET /users, /users/{id} | 利用者取得 |
| POST /users, PUT・DELETE /users/{id} | 管理者による利用者管理 |
| GET /resources, /resources/{id} | 設備取得 |
| POST /resources, PUT・DELETE /resources/{id} | 管理者による設備管理 |
| GET /reservations, /reservations/{id} | 予約取得 |
| POST /reservations | 予約作成 |
| PUT・DELETE /reservations/{id} | 予約別パスワードとversionで変更・取消 |
| GET /fullcalendar_events | カレンダー期間取得 |
| GET /settings | 共通予約設定 |
| PUT /settings | 管理者による設定更新 |
| POST /admin/login | 管理者ログイン |
| GET /admin/session | セッションとCSRFトークン取得 |
| POST /admin/logout | 管理者ログアウト |
| GET /admin/maintenance | 管理者向け運用情報 |
| GET /admin/backup | 管理者向けSQLiteバックアップ |
| GET /health | DB接続を含むヘルスチェック |

パラメーターとJSONの正確な形はsrc/modelとsrc/url_handlerを参照してください。画面はfrontend/src/api.tsを共通クライアントにしています。

管理者ログインはJSONとX-Reservoir-Request: 1が必要です。管理者の変更操作にはセッションCookieとX-CSRF-Tokenが必要です。CookieはHttpOnly / SameSite=Strict、セッションは8時間で失効します。予約のパスワードは予約の本人操作を確認するために使い、管理者認証とは別です。

予約の変更は単一UPDATE、通知イベントは同じトランザクションで保存します。SQLiteトリガーで同一設備の重複、日時、予約上限、参照整合性、終了済みや利用中の操作を保証します。履歴で参照される利用者・設備は削除できません。制約違反・競合は409、認証失敗は401、不正入力は400を返します。

新しいパスワードはArgon2idで保存し、旧SHA-256は正しいパスワードによる変更時に移行します。セッションはメモリー内に保持するため、サービス再起動時に管理者は再ログインします。Slackイベントは手動連絡の記録として保存し、自動送信ワーカーはまだ実装していません。

マイグレーションとバックアップの実運用についてはルートREADME.mdを参照してください。
