project('kime', 'c', 'cpp',
  version: files('VERSION'),
  meson_version: '>= 1.1',
  default_options: [
    'cpp_std=c++20',
    'buildtype=release',
    'b_ndebug=if-release',
    'c_args=-Wall -fvisibility=hidden',
    'cpp_args=-Wall -fvisibility=hidden',
  ],
)

cargo = find_program('cargo')
cargo_profile = get_option('cargo_profile')

if cargo_profile == 'release'
  cargo_args = ['--release']
  target_dir = meson.project_source_root() / 'target' / 'release'
else
  cargo_args = []
  target_dir = meson.project_source_root() / 'target' / 'debug'
endif

kime_engine_inc = include_directories('src/engine/capi')

if get_option('system_engine')
  message('system_engine mode: skipping cargo build, binary options (check, indicator, etc.) are ignored')
  # Use system-installed kime engine
  kime_engine_dep = declare_dependency(
    include_directories: kime_engine_inc,
    link_args: ['-lkime_engine'],
  )
else
  # Build engine and binaries from source via cargo
  cargo_packages = ['-p', 'kime-engine-capi']

  if get_option('check').allowed()
    cargo_packages += ['-p', 'kime-check']
  endif
  if get_option('indicator').allowed()
    cargo_packages += ['-p', 'kime-indicator']
  endif
  if get_option('candidate_window').allowed()
    cargo_packages += ['-p', 'kime-candidate-window']
  endif
  if get_option('xim').allowed()
    cargo_packages += ['-p', 'kime-xim']
  endif
  if get_option('wayland').allowed()
    cargo_packages += ['-p', 'kime-wayland']
  endif
  # kime main binary is always built when not using system engine
  cargo_packages += ['-p', 'kime']

  cargo_build = custom_target('cargo-build',
    command: ['sh', '-c', '"$@" && touch "@OUTPUT@"', '_',
              cargo, 'build'] + cargo_args + cargo_packages,
    output: 'cargo-build.stamp',
    build_always_stale: true,
    console: true,
    build_by_default: true,
  )

  kime_engine_dep = declare_dependency(
    include_directories: kime_engine_inc,
    link_args: ['-L' + target_dir, '-lkime_engine'],
    sources: cargo_build,
  )

  # Install cargo-built binaries
  meson.add_install_script('sh', '-c',
    'install -Dm755 "$1" -t "${DESTDIR}${MESON_INSTALL_PREFIX}/bin"',
    '_', target_dir / 'kime',
  )

  if get_option('check').allowed()
    meson.add_install_script('sh', '-c',
      'install -Dm755 "$1" -t "${DESTDIR}${MESON_INSTALL_PREFIX}/bin"',
      '_', target_dir / 'kime-check',
    )
  endif
  if get_option('indicator').allowed()
    meson.add_install_script('sh', '-c',
      'install -Dm755 "$1" -t "${DESTDIR}${MESON_INSTALL_PREFIX}/bin"',
      '_', target_dir / 'kime-indicator',
    )
  endif
  if get_option('candidate_window').allowed()
    meson.add_install_script('sh', '-c',
      'install -Dm755 "$1" -t "${DESTDIR}${MESON_INSTALL_PREFIX}/bin"',
      '_', target_dir / 'kime-candidate-window',
    )
  endif
  if get_option('xim').allowed()
    meson.add_install_script('sh', '-c',
      'install -Dm755 "$1" -t "${DESTDIR}${MESON_INSTALL_PREFIX}/bin"',
      '_', target_dir / 'kime-xim',
    )
  endif
  if get_option('wayland').allowed()
    meson.add_install_script('sh', '-c',
      'install -Dm755 "$1" -t "${DESTDIR}${MESON_INSTALL_PREFIX}/bin"',
      '_', target_dir / 'kime-wayland',
    )
  endif

  # Install engine shared library
  meson.add_install_script('sh', '-c',
    'install -Dm755 "$1" -t "${DESTDIR}${MESON_INSTALL_PREFIX}/@0@"'.format(get_option('libdir')),
    '_', target_dir / 'libkime_engine.so',
  )
endif

# Install headers
if get_option('install_headers')
  meson.add_install_script('sh', '-c',
    'install -Dm644 "$1" -t "${DESTDIR}${MESON_INSTALL_PREFIX}/include"',
    '_', meson.project_source_root() / 'src/engine/capi/kime_engine.hpp',
  )
  meson.add_install_script('sh', '-c',
    'install -Dm644 "$1" -t "${DESTDIR}${MESON_INSTALL_PREFIX}/include"',
    '_', meson.project_source_root() / 'src/engine/capi/kime_engine.h',
  )
endif

# Install desktop file
install_data('res/kime.desktop',
  install_dir: get_option('sysconfdir') / 'xdg' / 'autostart',
)
install_data('res/kime.desktop',
  install_dir: get_option('datadir') / 'applications',
)

# Install autostart helper script
meson.add_install_script('sh', '-c',
  'install -Dm755 "$1" -t "${DESTDIR}${MESON_INSTALL_PREFIX}/bin"',
  '_', meson.project_source_root() / 'res' / 'kime-xdg-autostart',
)

# Install icons
install_data(
  'res/icons/64x64/kime-hangul-black.png',
  'res/icons/64x64/kime-hangul-white.png',
  'res/icons/64x64/kime-latin-black.png',
  'res/icons/64x64/kime-latin-white.png',
  install_dir: get_option('datadir') / 'icons' / 'hicolor' / '64x64' / 'apps',
)

# Install docs
if get_option('install_docs')
  install_data(
    'res/default_config.yaml',
    'docs/CHANGELOG.md',
    'LICENSE',
    'NOTICE.md',
    'README.md',
    'README.ko.md',
    install_dir: get_option('datadir') / 'doc' / 'kime',
  )
endif

# Conditional frontend builds
if get_option('gtk3').allowed()
  subdir('src/frontends/gtk3')
endif
if get_option('gtk4').allowed()
  subdir('src/frontends/gtk4')
endif
if get_option('qt5').allowed()
  subdir('src/frontends/qt5')
endif
if get_option('qt6').allowed()
  subdir('src/frontends/qt6')
endif
