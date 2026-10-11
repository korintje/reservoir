# Reservoir frontend

React・TypeScript・Viteで構築した予約画面と管理者設定画面です。旧Create React Appのテンプレートを置き換えています。

```bash
npm ci
npm run dev
npm test
npm run build
npm run e2e
```

画面・API・管理者CookieはViteの公開パスに揃えます。例えば`npm run build -- --base /reservation/`なら、画面は`/reservation/`、APIは`/reservation/api/`です。React Routerも同じベースパスを使います。本番では`dist/`の内容をNginxのドキュメントルート下の`reservation/`に配置し、APIをバックエンド内部の`/api/`へ転送します。管理者Cookieは本番でHTTPSを必要とします。

開発時の既定パスは`/`です。`RESERVOIR_BASE_PATH=/reservation/ npm run dev`で公開パス配下の開発・`RESERVOIR_BASE_PATH=/reservation/ npm run e2e`でブラウザテストができます（PowerShellでは事前に`$env:RESERVOIR_BASE_PATH='/reservation/'`を設定）。ViteもAPI転送とCookieのパス変換を行います。

データ取得はTanStack Query、表示はFullCalendar、時刻はLuxonで日本時間へ統一します。月・週・日・一覧表示と設備の絞り込みを提供し、スマートフォンでは一覧を初期表示します。メンバー選択はURLの`user`クエリを切り替え、新規予約の初期メンバーにのみ使用します。

設定ページは管理者セッションとCSRFトークンを使用します。API失敗時にフォームの入力を保持し、予約のversionで他の端末からの変更を検出します。

Playwrightのブラウザーテストは専用の一時DBを使います。事前にbackend/でcargo build --lockedとnpx playwright install chromiumを実行してください。開発・導入・移行の手順はルートREADME.mdを参照してください。
