# rapt

Toy version of apt written in apt.  
  
めんどいのでガワだけなんとなく動くようにする。  
https://github.com/Debian/apt/blob/main/doc/apt-get.8.xml

# cache file
removeしたdebファイルは`/var/cache/apt/archives/`に保存されることがあり、reinstall時に再度ダウンロードしなくてすむようにキャッシュ。
aptのキャッシュファイルは`/var/cache/apt/pkgcache.bin`.こいつが何してるかは正確にはわかってないが、まぁ多分Packagesの中身(且つpriority等を解決した結果)をシリアライズして入れることで計算の手間を省いてそう。raptではめんどいので毎回fetchするし毎回計算する。
# ソースリスト
ソースリストは`/etc/sources.list`もしくは`/etc/sources.list.d`以下のファイル。`update`時にこれらのリストのリポジトリからメタデータをfetchしてきて`/var/cache/apt/pkgccache.bin`に入れる。  
フォーマット: https://people.debian.org/~jak/apt2-doc/apt-Cache-Format.html  

このindex(Packagesは`/var/lib/apt/lists`以下に保存される)。  
実際にインストールされたパッケージ一覧は`/var/lib/dpkg/status`に保存される。
  
# Source Package
format: https://wiki.debian.org/Packaging/SourcePackage?action=show&redirect=SourcePackage

## priority
required, important, standard, optional, extra。ディストロに詰めるパッケージを示してるっぽいからaptでは関係なし。

## sections
これもまあ関係なし。 https://packages.debian.org/unstable/

## Package-List
そのソースパッケージからビルドできるパッケージ一覧っぽい。

## Files
`Directory`以下に存在する対象ファイル一覧。基本はファイル本体のtarballとdscファイルなのかな。
```.txt
[space][MD5 checksum][space][size][space][filename]
```

## Priority
`required`の場合削除不可。 

## Pre-Depends
https://www.debian.org/doc/debian-policy/ch-relationships.html

## Tasks
なんやねんこれ

## Description
複数行になり得る(leading space)

# .deb
以下の3つのfile/dirがarアーカイブされている。
- debian-binary: パッケージのバージョン(2.0 now)
- data archive: 実際にインストールするファイル群。
- control archive: pre/postスクリプト等。
  - この中の`control`がindexのentryと同じ内容を持っていて、dependency解決できる。