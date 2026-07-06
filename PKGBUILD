# Maintainer: Aner <argent0@github.com>
pkgname=bodydashboard
pkgver=0.1.0
pkgrel=1
pkgdesc="Generate a mobile-first HTML health dashboard from bodylog, sleep, and nutlog data"
arch=('x86_64')
url="https://github.com/argent0/bodydashboard"
license=('custom')
depends=('gcc-libs' 'bodylog' 'nutlog')
provides=('bodydashboard')
makedepends=('git' 'rust' 'cargo')
source=("${pkgname}::git+ssh://git@github.com/argent0/bodydashboard.git")
sha256sums=('SKIP')

pkgver() {
  cd "$srcdir/$pkgname"
  local _ver=$(grep '^version =' Cargo.toml | head -n 1 | cut -d '"' -f 2)
  echo "${_ver}.r$(git rev-list --count HEAD).$(git rev-parse --short HEAD)"
}

build() {
  cd "$srcdir/$pkgname"
  cargo build --release --locked
}

package() {
  cd "$srcdir/$pkgname"
  install -Dm755 "target/release/bodydashboard" "$pkgdir/usr/bin/bodydashboard"
  install -Dm644 "prompt.md" "$pkgdir/usr/share/doc/$pkgname/prompt.md"
}