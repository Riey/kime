qt6_mod = import('qt6')
qt6_dep = dependency('qt6', modules: ['Core', 'Gui'], private_headers: true, required: get_option('qt6'))

if qt6_dep.found()
  qt6_plugindir_opt = get_option('qt6_plugindir')
  if qt6_plugindir_opt != ''
    qt6_plugindir = qt6_plugindir_opt
  else
    qmake6 = find_program('qmake6', 'qmake-qt6', required: false)
    if qmake6.found()
      qt6_plugindir = run_command(qmake6, '-query', 'QT_INSTALL_PLUGINS', check: true).stdout().strip()
    else
      qt6_plugindir = get_option('prefix') / get_option('libdir') / 'qt6' / 'plugins'
    endif
  endif

  qt6_moc = qt6_mod.preprocess(
    moc_headers: ['../qt5/src/plugin.hpp', '../qt5/src/input_context.hpp'],
    dependencies: [qt6_dep, kime_engine_dep],
    moc_extra_arguments: ['-DKIME_QT_IID="org.qt-project.Qt.QPlatformInputContextFactoryInterface.5.1"'],
  )

  shared_library('kimeplatforminputcontextplugin',
    '../qt5/src/plugin.cc',
    '../qt5/src/input_context.cc',
    qt6_moc,
    dependencies: [qt6_dep, kime_engine_dep],
    cpp_args: ['-DKIME_QT_IID="org.qt-project.Qt.QPlatformInputContextFactoryInterface.5.1"'],
    install: true,
    install_dir: qt6_plugindir / 'platforminputcontexts',
  )
endif
