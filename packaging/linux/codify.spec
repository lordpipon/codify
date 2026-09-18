# RPM spec for Codify (build with: rpmbuild -bb codify.spec)
%global debug_package %{nil}

Name:           codify
Version:        0.1.2
Release:        1%{?dist}
Summary:        A fast, minimal IDE built with GPUI

License:        MIT
URL:            https://github.com/lordpipon/codify
Source0:        https://github.com/lordpipon/codify/archive/%{version}.tar.gz

BuildRequires:  cargo, rust >= 1.85, gcc, clang, make, pkgconfig
Requires:       fontconfig, freetype, libxkbcommon

%description
Codify is a fast, minimal IDE built on Rust + GPUI (the same engine that
powers Zed). It ships with a file explorer, syntax highlighting for dozens of
languages, an integrated terminal, and light/dark/system theming with accent
colours.

%prep
%autosetup -n codify-%{version}

%build
cargo build --release --locked

%install
install -Dm755 %{_builddir}/%{name}-%{version}/target/release/codify %{buildroot}%{_bindir}/codify
install -Dm644 %{_builddir}/%{name}-%{version}/packaging/linux/codify.desktop %{buildroot}%{_datadir}/applications/org.codify.Codify.desktop
for s in 16 24 32 48 64 128 256 512 1024; do
  install -Dm644 %{_builddir}/%{name}-%{version}/packaging/linux/icons/hicolor/${s}x${s}/apps/org.codify.Codify.png \
    %{buildroot}%{_datadir}/icons/hicolor/${s}x${s}/apps/org.codify.Codify.png
done
mkdir -p %{buildroot}%{_datadir}/licenses/%{name}
cp %{_builddir}/%{name}-%{version}/LICENSE %{buildroot}%{_datadir}/licenses/%{name}/LICENSE

%files
%{_bindir}/codify
%{_datadir}/applications/org.codify.Codify.desktop
%{_datadir}/icons/hicolor/*/apps/org.codify.Codify.png
%{_datadir}/licenses/%{name}/LICENSE

%changelog
* Thu Sep 17 2026 lordpipon <lordpipon@users.noreply.github.com> - 0.1.0-1
- Initial release