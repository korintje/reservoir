# Reservoir

研究室の設備を予約するための、小規模な予約管理システムです。ブラウザーから予約の閲覧・作成・変更・取消を行い、管理者が利用者・設備・予約ルールを設定します。

## できること

- PC向けの日・週・月カレンダー、スマートフォン向けの一覧、設備・利用者による絞り込み。
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

アプリは専用ユーザーで127.0.0.1:8080に待ち受けます。NginxがHTTPSと静的画面を提供し、/apiをそのまま転送します。WebプロセスにはOS管理権限を与えません。インストーラーは自分が付けたUFWのWeb許可ルールを更新し、他のアプリのルールは削除しません。

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

不整合な予約期間を推定補正する場合は、ウィザードの「Auto-assume」でyesを選ぶか、以下のオプションを使います。既定はOFFです。

```bash
sudo bash install.sh --auto-assume
# 初回の旧DB取り込みと組み合わせる場合
sudo bash install.sh --import-db /path/to/reservoir-export.db --auto-assume
```

サービス停止中にDB全体をバックアップし、開始が終了より後なら反転、同時刻なら削除します。補正・削除前のレコードはreservation_repair_historyにも保存します。補正した日時が本来の意図と一致する保証はありません。補正後の時間重複・参照先欠落・日時範囲の検査は引き続き実施するため、別の不整合があれば起動を拒否します。すでに導入失敗後のDBがPiにある場合は--import-dbを再指定せず--auto-assumeで再実行してください。

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
