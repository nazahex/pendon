### Demo

```sh
lokat build

› Scanning source files...
  - Found: 45 files

› Extracting keys...
  - Existing: 127 keys
  - Found: 23 new keys

✖ ⟦error⟧
  - Invalid syntax at line 134
  - Missing keys at line 142

▲ ⟦warn⟧ Unused keys detected:
  - home.old_banner (2 keys)
    - brand
    - slogan
  - promo.summer_2024 (3 keys)
    - gift.dummy
    - tagline
    - description

✔ ⟦done⟧
  - Updated locales/en.json (+23 keys)

◆ ⟦info⟧
  - New keys (need translation):
    - dashboard.stats.users
    - dashboard.stats.revenue
    - settings.account.email
    ... 20 more
```

### Format

- `✔ ⟦done⟧` green bold
- `◆ ⟦info⟧` blue bold
- `▲ ⟦warn⟧` yellow bold
- `✖ ⟦error⟧` red bold
