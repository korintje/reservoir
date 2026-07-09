# reservoir

Rust、Actix Web、SQLiteで作られた、設備や部屋などのリソースを予約するためのWeb APIです。

## 主な機能

- 利用者（user）の登録、取得、更新、削除
- 予約対象（resource）の登録、取得、更新、削除
- 予約（reservation）の登録、検索、更新、削除
- FullCalendarのイベント形式による予約一覧の取得

## 必要な環境

- Rust 2021 Editionに対応したRustツールチェーン（Cargoを含む）

## 起動方法

リポジトリのルートで次のコマンドを実行します。

```console
cargo run
```

デフォルトでは `http://127.0.0.1:8080` でAPIが起動します。起動時にSQLiteのテーブルが存在しなければ自動的に作成されます。

接続先とデータベースのパスは `reservoir.toml` で変更できます。

```toml
web_url = "127.0.0.1:8080"
db_path = "./db/reservoir.db"
```

設定ファイルとデータベースの相対パスは、プロセスのカレントディレクトリを基準に解決されます。そのため、通常はリポジトリのルートから起動してください。

## API概要

ベースURLは、デフォルト設定の場合 `http://127.0.0.1:8080` です。リクエストおよびレスポンスの本文にはJSONを使用します。

| メソッド | パス | 説明 |
| --- | --- | --- |
| `GET` | `/users` | 利用者一覧を取得 |
| `GET` | `/users/{id}` | 利用者を1件取得 |
| `POST` | `/users` | 利用者を登録 |
| `PUT` | `/users/{id}` | 利用者を更新 |
| `DELETE` | `/users/{id}` | 利用者を削除 |
| `GET` | `/resources` | リソース一覧を取得 |
| `GET` | `/resources/{id}` | リソースを1件取得 |
| `POST` | `/resources` | リソースを登録 |
| `PUT` | `/resources/{id}` | リソースを更新 |
| `DELETE` | `/resources/{id}` | リソースを削除 |
| `GET` | `/reservations` | 条件に合う予約一覧を取得 |
| `GET` | `/reservations/{id}` | 予約を1件取得 |
| `POST` | `/reservations` | 予約を登録 |
| `PUT` | `/reservations/{id}` | 予約を更新 |
| `DELETE` | `/reservations/{id}` | 予約を削除 |
| `GET` | `/fullcalendar_events` | FullCalendar形式で予約一覧を取得 |

日時はRFC 3339形式で指定します。たとえば、日本時間の2026年7月10日9時は `2026-07-10T09:00:00+09:00` です。

### 共通レスポンス

登録、更新、削除に成功すると次のレスポンスを返します。

```json
{
  "code": 2000,
  "message": "Success"
}
```

代表的なエラーは次のとおりです。

| HTTPステータス | `code` | 内容 |
| --- | ---: | --- |
| `400 Bad Request` | `4000` | 入力値またはデータベース操作が不正 |
| `401 Unauthorized` | `4001` | 予約のパスワードが不一致 |
| `404 Not Found` | `4004` | 対象が存在しない |
| `500 Internal Server Error` | `5000` | サーバー内部のエラー |

JSONの構文や型が不正な場合は、`400 Bad Request` と `{"error":"..."}` を返します。

## 利用者API

利用者は次のフィールドを持ちます。

| フィールド | 型 | 説明 |
| --- | --- | --- |
| `id` | integer or null | 利用者ID。登録時は省略可能 |
| `user_name` | string or null | 利用者名。登録時は必須 |
| `active` | boolean or null | 有効状態。登録時の省略値は `true` |

### 一覧・1件取得

```http
GET /users
GET /users/{id}
```

一覧は利用者オブジェクトの配列、1件取得は利用者オブジェクトを返します。

### 登録

```http
POST /users
Content-Type: application/json

{
  "user_name": "山田太郎",
  "active": true
}
```

### 更新

```http
PUT /users/{id}
Content-Type: application/json

{
  "user_name": "山田花子",
  "active": false
}
```

指定したフィールドだけを更新します。`id` はリクエスト本文ではなくURLの値が使われます。

### 削除

```http
DELETE /users/{id}
```

## リソースAPI

リソースは次のフィールドを持ちます。

| フィールド | 型 | 説明 |
| --- | --- | --- |
| `id` | integer or null | リソースID。登録時は省略可能 |
| `resource_name` | string or null | リソース名。登録時は実質必須 |
| `active` | boolean or null | 有効状態。登録時は実質必須 |
| `capacity` | integer or null | 定員。登録時は実質必須 |
| `custom_field` | string or null | 任意情報。登録時は実質必須 |

`POST /resources` ではSQLiteの各列が `NOT NULL` のため、`id` 以外のフィールドをすべて指定してください。

### 一覧・1件取得

```http
GET /resources
GET /resources/{id}
```

一覧はリソースオブジェクトの配列、1件取得はリソースオブジェクトを返します。

### 登録

```http
POST /resources
Content-Type: application/json

{
  "resource_name": "会議室A",
  "active": true,
  "capacity": 8,
  "custom_field": "3階"
}
```

### 更新

```http
PUT /resources/{id}
Content-Type: application/json

{
  "capacity": 10,
  "custom_field": "3階・モニターあり"
}
```

指定したフィールドだけを更新します。`id` はリクエスト本文ではなくURLの値が使われます。

### 削除

```http
DELETE /resources/{id}
```

## 予約API

### 一覧取得

```http
GET /reservations
```

次のクエリーパラメーターを任意で組み合わせられます。

| パラメーター | 型 | 条件 |
| --- | --- | --- |
| `from` | RFC 3339 datetime | 予約終了日時がこの値より後 |
| `until` | RFC 3339 datetime | 予約開始日時がこの値以前 |
| `user_id` | integer | 利用者IDが一致 |
| `resource_id` | integer | リソースIDが一致 |

`from` と `until` を指定すると、その期間と重なる予約を取得できます。URL中の `+` は `%2B` にエンコードしてください。

```http
GET /reservations?from=2026-07-10T00%3A00%3A00%2B09%3A00&until=2026-07-11T00%3A00%3A00%2B09%3A00&resource_id=1
```

レスポンス例:

```json
[
  {
    "id": 1,
    "resource_id": 1,
    "resource_name": "会議室A",
    "user_id": 1,
    "user_name": "山田太郎",
    "start": "2026-07-10T09:00:00+09:00",
    "end": "2026-07-10T10:00:00+09:00",
    "description": "週次ミーティング",
    "created_at": "2026-07-09T12:00:00+09:00"
  }
]
```

### 1件取得

```http
GET /reservations/{id}
```

### 登録

```http
POST /reservations
Content-Type: application/json

{
  "resource_id": 1,
  "user_id": 1,
  "start": "2026-07-10T09:00:00+09:00",
  "end": "2026-07-10T10:00:00+09:00",
  "description": "週次ミーティング",
  "password": "secret"
}
```

`resource_id`、`user_id`、`start`、`end` は必須です。`description` と `password` は省略または `null` にできます。開始日時は終了日時より前でなければなりません。

パスワードそのものは保存されず、SHA-256ハッシュが保存されます。パスワードを省略した予約は空文字列をパスワードとして扱います。

### 更新

```http
PUT /reservations/{id}
Content-Type: application/json

{
  "password": "secret",
  "start": "2026-07-10T10:00:00+09:00",
  "end": "2026-07-10T11:00:00+09:00",
  "description": "開始時刻変更",
  "new_password": "new-secret"
}
```

`password` には現在のパスワードを指定し、更新したいフィールドだけを加えます。更新できるフィールドは `resource_id`、`user_id`、`start`、`end`、`description` です。`new_password` を指定するとパスワードも変更できます。

### 削除

`DELETE` でもJSON本文に現在のパスワードを指定します。

```http
DELETE /reservations/{id}
Content-Type: application/json

{
  "password": "secret"
}
```

パスワードなしで登録した予約では、`password` を `null` にするか空文字列を指定します。

### FullCalendar形式

```http
GET /fullcalendar_events
```

任意のクエリーパラメーターとして `start`、`end`、`resource_id` を指定できます。`start` と `end` はRFC 3339形式です。

レスポンス例:

```json
[
  {
    "id": 1,
    "title": "山田太郎 (週次ミーティング)",
    "start": "2026-07-10T09:00:00+09:00",
    "end": "2026-07-10T10:00:00+09:00",
    "description": "週次ミーティング"
  }
]
```

## ターミナルからAPIを利用する例

以下はAPIを起動した状態で、別のターミナルから `curl` を実行する一連の例です。Windows PowerShellでは、組み込みの別名との衝突を避けるため `curl.exe` と読み替えられます。

### 1. 利用者を登録

```console
curl -X POST http://127.0.0.1:8080/users \
  -H "Content-Type: application/json" \
  -d '{"user_name":"山田太郎","active":true}'
```

### 2. リソースを登録

```console
curl -X POST http://127.0.0.1:8080/resources \
  -H "Content-Type: application/json" \
  -d '{"resource_name":"会議室A","active":true,"capacity":8,"custom_field":"3階"}'
```

### 3. 登録されたIDを確認

```console
curl http://127.0.0.1:8080/users
curl http://127.0.0.1:8080/resources
```

以降の例では、利用者とリソースのIDがともに `1` だったものとします。

### 4. 予約を登録

```console
curl -X POST http://127.0.0.1:8080/reservations \
  -H "Content-Type: application/json" \
  -d '{"resource_id":1,"user_id":1,"start":"2026-07-10T09:00:00+09:00","end":"2026-07-10T10:00:00+09:00","description":"週次ミーティング","password":"secret"}'
```

### 5. 予約を検索

`curl -G --data-urlencode` を使うと、日時の `+` などを安全にURLエンコードできます。

```console
curl -G http://127.0.0.1:8080/reservations \
  --data-urlencode "from=2026-07-10T00:00:00+09:00" \
  --data-urlencode "until=2026-07-11T00:00:00+09:00" \
  --data-urlencode "resource_id=1"
```

### 6. 予約を更新

```console
curl -X PUT http://127.0.0.1:8080/reservations/1 \
  -H "Content-Type: application/json" \
  -d '{"password":"secret","description":"議題追加"}'
```

### 7. 予約を削除

```console
curl -X DELETE http://127.0.0.1:8080/reservations/1 \
  -H "Content-Type: application/json" \
  -d '{"password":"secret"}'
```

## 実装上の注意

- 認証機構はなく、CORSはすべてのオリジン、メソッド、ヘッダーを許可しています。公開ネットワークで利用する場合は、認証、アクセス制御、TLSなどを別途用意してください。
- 利用者やリソースの `active` は情報として保存されますが、現在の実装では無効な利用者・リソースによる予約を拒否しません。
- 予約時間の重複やリソースの定員は検証されません。
- 予約の更新時には、開始・終了日時の前後関係や、指定した利用者・リソースの存在は検証されません。
- 利用者またはリソースを削除すると、それを参照する予約は一覧・詳細取得の結合結果に現れなくなります。

## ライセンス

[MIT License](LICENSE)
