# Reservoir

研究室の設備を予約するための、小規模な予約管理システムです。ブラウザーから予約の閲覧・作成・変更・取消を行い、管理者が利用者・設備・予約ルールを設定します。

## できること

- PC向けの日・週・月カレンダー、スマートフォン向けの一覧、設備による絞り込み。URLの`user`クエリで新規予約の初期メンバーを指定。
- 同一設備の時間重複禁止、設定可能な予約上限（初期15日）、終了済み予約の変更・取消禁止。
- 共通Basic認証のON/OFF、独立した管理者ログイン、予約別パスワード。
- 利用者・設備の追加・編集・無効化、Slack連絡先、研究室名と案内文の設定。
- 予約変更の競合検出、手動Slack連絡の案内、定期バックアップと復元。

画面・操作メッセージは英語、予約の表示・入力は日本時間です。データはUTCで保存します。設備の定員・台数は参考情報であり、値に関係なく同時予約は1件です。

## Raspberry Piへ導入

想定環境はRaspberry Pi 4、64bitのUbuntu Server 24.04以降、インターネット接続、sudo権限です。ビルドのため4GB以上の空きディスクが必要です。ソースのビルドには時間がかかります。32bit OSは対象外です。

このリポジトリーをPiへコピーして、そのディレクトリーで実行します。開発用node_modulesやtarget、既存DB、秘密設定はコピー不要です。

```bash
bash install.sh --plan             # OSを変更せず導入内容を確認
sudo bash install.sh              # 対話式に導入
```

ウィザードが次を設定します。

1. LAN限定 / インターネット＋LAN、アクセス名またはIP、LANの許可ネットワーク。
2. HTTPS（自己署名、既存証明書、ACME）、共通Basic認証、管理者パスワード。
3. SSHポートの通信許可、時刻同期、OSタイムゾーン、自動セキュリティ更新。
4. 必要パッケージ、RustとNode.js、専用ユーザーによるビルド、Nginx、systemd。
5. 毎日のバックアップと保持期間、動作確認、任意の固定IP設定。

アプリは専用ユーザーで127.0.0.1:8080に待ち受けます。NginxがHTTPSと静的画面を提供します。公開パスが`/reservation/`の場合、画面は`https://ホスト名/reservation/`、APIは`/reservation/api/`、設定画面は`/reservation/settings`です。NginxはAPIをバックエンド内部の`/api/`へ転送し、管理者Cookieのパスも書き換えます。WebプロセスにはOS管理権限を与えません。インストーラーは自分が付けたUFWのWeb許可ルールを更新し、他のアプリのルールは削除しません。

ウィザードの「公開パス」で`/reservation/`などを指定できます。新規導入の既定は`/reservation/`、従来の配置を更新する場合は`/`を維持します。LAN・internetどちらでも利用でき、末尾の`/`は自動補完します。`/reservation?user=1`はクエリを維持して`/reservation/?user=1`へ転送されます。ビルド成果物は`/opt/reservoir/current/frontend/reservation/`へ配置します。`/lab/reservation/`など複数階層も利用できます。公開パスの変更には`sudo bash install.sh`を再実行してください。再ビルドが必要なため、`reservoirctl configure`は公開パスを維持します。

「IPアドレス直打ちを拒否する」をONにすると、IPv4・IPv6のIPをHostとして受けたHTTP/HTTPSリクエストをNginxの`return 444`で切断します。既定はOFFで、ON時はアクセス先にIPではなくホスト名を指定する必要があります。導入後も`sudo reservoirctl configure`で変更できます。HTTPSではTLS接続後に切断するため、IPでのアクセスでは先に証明書警告が出る場合があります。接続先IPを隠す機能ではなく、ホスト名を明示した通信は許可します。他のサイトに明示的なIP用`server_name`がある場合、そのサイトの設定が優先されます。

フロントエンドだけ更新する場合も、保存済み公開パスに合わせてビルドし、同じディレクトリに配置してください（例: `npm run build -- --base /reservation/`）。`dist/`の中身を`frontend/reservation/`へ配置し、`dist/`そのものを下に入れないでください。

LANでは現在のIPが入力候補になります。独自のホスト名を使う場合は研究室DNS等で名前解決を設定してください。自己署名証明書では利用端末への信頼設定が必要です。証明書の期限は825日です。

インターネット公開では公開ドメイン名と既存証明書またはACMEを選択します。DNS、大学側の公開許可、ルーターの転送などPiの外側の設定は別途必要です。ACMEのHTTP検証には外部から80番ポートへ到達できる必要があります。取得後の更新とNginxの再読み込みはCertbotで行います。LAN限定では自己署名または既存証明書を使います。インターネット公開からLAN限定へ切り替えて既存証明書を引き継ぐ場合、その後の証明書更新はDNS検証等で別途管理してください。

固定IPは最後にNetplanの一時適用で確認します。アクセス用IPを変更する場合は公開設定と証明書の名前も合わせて変更してください。ネットワーク設定中にSSHが切れた場合はPiのコンソールから確認できます。

## 旧VPSからのデータ移行

**稼働中のDBファイルを単純にコピーせず、SQLiteのバックアップ機能でコピーを作成してください。**

VPSで予約更新を停止するか、SQLiteの整合したスナップショットを取得します。

```bash
sqlite3 /path/to/reservoir.db ".backup '/path/to/reservoir-export.db'"
```

取得したコピーをPiへ転送し、初回導入時に指定します。

```bash
sudo bash install.sh --import-db /path/to/reservoir-export.db
```

既存の利用者・設備・予約・旧パスワードハッシュを維持してマイグレーションします。旧SHA-256ハッシュは、正しい予約パスワードで変更・取消したときにArgon2へ更新されます。パスワードを変更して保存することもできます。

DB不整合で起動が拒否された場合は、インストーラーとは別の対話式修復スクリプトを実行します。

```bash
sudo bash repair-db.sh
# 修復後、導入に失敗していた場合は再実行（--import-dbは再指定しない）
sudo bash install.sh
```

開始 > 終了の予約は、実行時に「削除」または「開始・終了を反転」を選択します。開始 = 終了は削除します。同じ設備の完全重複は大きいIDを残し、内包重複は長い予約を残して短い予約を削除します（開始または終了だけが同じ場合も含む）。その後、部分重複は後の予約の開始を前の予約の終了に揃えます。隣接する予約や別設備の予約は対象外です。

サービスを停止し、変更予定を表示します。`REPAIR`と入力するとDB全体をバックアップして一括修復します。修復前のレコードは`reservation_repair_history`にも保存します。未解決の日時範囲・参照先欠落などがあれば修復全体を取り消します。中止・失敗時もサービスは停止状態です。導入済みの場合は修復後に`sudo systemctl start reservoir`で起動してください。DBとバックアップ先は`--db /path/to/db --backup-dir /path/to/backups`で指定できます。インストーラー自体はDBを自動修復しません。

開始と終了の逆転、不正な日時、参照先の欠落、同一設備の重複がある旧DBは起動前に拒否します。元データは自動削除・修正しません。バックアップ上で該当予約を確認してから移行をやり直してください。更新時は移行前のバックアップを保存し、アプリ切替に失敗した場合は旧DBと旧リリースへ戻します。

Piで設備・利用者・日時・変更と取消を確認した後、利用者へ新しいアクセス先を案内してVPS側への書き込みを止めます。

## 日常運用

ブラウザーの「設定」に管理者パスワードでログインし、設備・利用者と予約ルールを管理します。履歴がある設備・利用者は削除せず無効化してください。管理者にも終了済み予約の変更や予約パスワードの迂回は設けていません。

公開範囲やBasic認証の変更はサーバーで行います。

```bash
sudo reservoirctl configure
sudo reservoirctl status
sudo reservoirctl backup
sudo reservoirctl restore /var/backups/reservoir/reservoir-....db
sudo journalctl -u reservoir -n 100 --no-pager
```

restoreは現在のDBを事前保存し、確認入力後にサービスを停止して復元します。バックアップには利用者情報とパスワードハッシュが含まれます。サーバーの故障に備えてバックアップを別の機器にも保管してください。

設定画面の保守タブからもDBをダウンロードできます。定期バックアップは初期設定で毎日03:00、30日保持です。保守タブに最終実行の成功・失敗を表示します。「手動連絡対象の記録」は対象操作の件数であり、Slackへ連絡済みかどうかは判定しません。

更新は新しいソースでsudo bash install.shを再実行します。入力候補は保存した公開設定を引き継ぎます。OSパッケージの導入や自動更新設定まで元に戻す処理は行いません。途中で失敗した場合は表示された段階とログを確認し、同じスクリプトを再実行してください。

| 配置 | 内容 |
| --- | --- |
| /opt/reservoir/releases/ | ビルド済みリリース |
| /opt/reservoir/current | 使用中リリースへのリンク |
| /etc/reservoir/ | アプリ・公開設定・認証情報 |
| /var/lib/reservoir/reservoir.db | 本番DB |
| /var/backups/reservoir/ | 初期設定のバックアップ保存先 |

## 予約と認証の仕様

予約上限は現在時刻からn×24時間後までです。開始・終了とも上限内で、終了は開始より後である必要があります。境界が隣接する予約は重複に含めません。

利用中の予約は開始日時の変更と取消ができません。終了済みは変更・取消できません。開始まで24時間以内の開始延期・終了繰上げ・取消、利用中の終了繰上げは保存後にSlack手動連絡を案内します。操作と同じトランザクションで通知イベントを記録します。現在はSlackへ自動送信しません。設備ごとのチャネルIDを保持し、後から通知処理を追加できます。

共通Basic認証をOFFにすると、アクセス可能な人が予約を閲覧・作成できます。予約パスワードは任意のため、パスワードなし予約はアクセス可能な人が変更・取消できます。管理者認証はBasicのON/OFFに関係なく必須です。Basic認証のON/OFFはNginxが実際のアクセスを制御し、アプリ設定は状態表示に使います。

## 開発と検証

必要な環境はRust、Node.js 24 LTS、Python 3です。依存はCargo.lockとpackage-lock.jsonで固定します。

```bash
cd backend
cargo build --locked
cd ../frontend
npm ci
cd ..
python scripts/dev.py
```

初回は開発用管理者パスワードを入力します。開発データは.local/に保存し、同梱の旧DBや本番DBには触れません。画面はhttp://127.0.0.1:5173、APIは8080です。HTTPでの開発に限りSecure Cookieを無効にします。

```bash
cd backend
cargo fmt --check
cargo clippy --all-targets --locked -- -D warnings
cargo test --locked
cargo build --locked
cd ../frontend
npm test
npm run build
npm audit --audit-level=low
npx playwright install chromium
npm run e2e
cd ../deploy
python3 -m unittest discover -v
```

E2Eは一時DBで管理者ログイン、設備・利用者追加、予約、重複拒否、パスワード変更操作、取消、設定変更、PC・スマートフォン表示を検証します。8080と5173が空いている必要があります。実行中のサーバーを止めてからRustを再ビルドしてください。

UbuntuでのOS設定・systemd・証明書発行を含むインストーラーの通し実行はPi実機での確認が必要です。ローカルの検証結果と未確認項目はIMPLEMENTATION_STATUS.mdに記載しています。

## 構成

- frontend/: React / TypeScript / Vite / FullCalendar。
- backend/: Actix Web / SQLite / sqlx migrations。
- deploy/: Ubuntu導入、Nginx設定生成、運用CLI、systemd。
- scripts/: 開発起動とブラウザーテスト用サーバー。

ライセンスはbackend/LICENSEを参照してください。
