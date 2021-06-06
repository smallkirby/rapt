# rapt

Toy version of apt written in apt.  
  
めんどいのでガワだけなんとなく動くようにする。  
https://github.com/Debian/apt/blob/main/doc/apt-get.8.xml

# cache file
aptのキャッシュファイルは`/var/cache/apt/pkgcache.bin`.
```.sh
$ file /var/cache/apt/pkgcache.bin
/var/cache/apt/pkgcache.bin: APT cache data, version 16.0, little-endian, 121553 packages, 120925 versions
```

# ソースリスト
ソースリストは`/etc/sources.list`もしくは`/etc/sources.list.d`以下のファイル。`update`時にこれらのリストのリポジトリからメタデータをfetchしてきて`/var/cache/apt/pkgccache.bin`に入れる。  
フォーマット: https://people.debian.org/~jak/apt2-doc/apt-Cache-Format.html

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