Name: ski
Version: 6.4
Release: 1
URL: http://www.catb.org/~esr/ski/
Source0: %{name}-%{version}.tar.gz
License: GPL
Group: Games
Summary: a hotdogging game, evade the deadly Yeti on your jet-powered skis
BuildRoot: %{_tmppath}/%{name}-root
BuildArch: noarch
#Keywords: game, solitaire

%description
Imagine you are skiing down an infinite slope, facing such hazards as
trees, ice, bare ground, and the man-eating Yeti!  Unfortunately,
you have put your jet-powered skis on backwards, so you can't see
ahead where you are going; only behind where you have been.  However,
you can turn to either side, jump or hop through the air, teleport
through hyperspace, launch nuclear ICBMs, and cast spells to call the
Fire Demon.  And since the hazards occur in patches, you can
skillfully outmaneuver them.  A fun and very silly game that proves
you don't need fancy graphical user interfaces to have a good time.

%prep 
%setup -q

%build
make %{?_smp_mflags} ski.6

%install
[ "$RPM_BUILD_ROOT" -a "$RPM_BUILD_ROOT" != / ] && rm -rf "$RPM_BUILD_ROOT"
mkdir -p "$RPM_BUILD_ROOT"%{_bindir}
mkdir -p "$RPM_BUILD_ROOT"%{_mandir}/man6/
cp ski "$RPM_BUILD_ROOT"%{_bindir}
cp ski.6 "$RPM_BUILD_ROOT"%{_mandir}/man6/

%clean
[ "$RPM_BUILD_ROOT" -a "$RPM_BUILD_ROOT" != / ] && rm -rf "$RPM_BUILD_ROOT"

%files
%doc README COPYING
%defattr(-,root,root,-)
%{_mandir}/man6/ski.6*
%{_bindir}/ski

%changelog
* Sun Jan 11 2004 Eric S. Raymond <esr@snark.thyrsus.com> 6.4-1
- Game commands now accept numeric repeat prefixes.

* Sat Jan  3 2004 Eric S. Raymond <esr@snark.thyrsus.com> 6.3-1
- Fix bug introduced to teleport command in 6.2.

* Tue Dec 31 2003 Eric S. Raymond <esr@snark.thyrsus.com> 6.2
- Emit reset on keyboard interrupt, just in case user bails out in the
  middle af a row refresh.  Fix bug in ! command interpretation.
  Terrain key is now colorized.

* Tue Dec 30 2003 Eric S. Raymond <esr@snark.thyrsus.com> 6.1
- Python reimplementation of the original C game.


