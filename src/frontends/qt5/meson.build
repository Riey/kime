qt5_mod = import('qt5')
qt5_dep = dependency('qt5', modules: ['Core', 'Gui'], private_headers: true, required: get_option('qt5'))

if qt5_dep.found()
  qt5_plugindir_opt = get_option('qt5_plugindir')
  if qt5_plugindir_opt != ''
    qt5_plugindir = qt5_plugindir_opt
  else
    qmake5 = find_program('qmake-qt5', 'qmake5', 'qmake', required: false)
    if qmake5.found()
      qt5_plugindir = run_command(qmake5, '-query', 'QT_INSTALL_PLUGINS', check: true).stdout().strip()
    else
      qt5_plugindir = get_option('prefix') / get_option('libdir') / 'qt5' / 'plugins'
    endif
  endif

  qt5_moc = qt5_mod.preprocess(
    moc_headers: ['src/plugin.hpp', 'src/input_context.hpp'],
    dependencies: [qt5_dep, kime_engine_dep],
  )

  shared_library('kimeplatforminputcontextplugin',
    'src/plugin.cc',
    'src/input_context.cc',
    qt5_moc,
    dependencies: [qt5_dep, kime_engine_dep],
    install: true,
    install_dir: qt5_plugindir / 'platforminputcontexts',
  )
endif
