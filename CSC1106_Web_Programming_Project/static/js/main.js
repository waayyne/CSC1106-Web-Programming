document.addEventListener('DOMContentLoaded', function () {
	const sidebarToggle = document.getElementById('sidebarToggle');
	const sidebar = document.getElementById('sidebar');
	const mainContent = document.getElementById('mainContent');
	const sidebarDropdowns = document.querySelectorAll('.nav-dropdown-title');

	if (!sidebarToggle || !sidebar || !mainContent) {
		return;
	}

	const syncSidebarState = function (isCollapsed) {
		sidebar.classList.toggle('sidebar-collapsed', isCollapsed);
		mainContent.classList.toggle('main-content-collapsed', isCollapsed);
		sidebarToggle.setAttribute('aria-expanded', String(!isCollapsed));
	};

	syncSidebarState(sidebar.classList.contains('sidebar-collapsed'));

	sidebarToggle.addEventListener('click', function () {
		syncSidebarState(!sidebar.classList.contains('sidebar-collapsed'));
	});

	sidebarDropdowns.forEach(function (summary) {
		summary.addEventListener('click', function (event) {
			if (!sidebar.classList.contains('sidebar-collapsed')) {
				return;
			}

			event.preventDefault();
			syncSidebarState(false);

			const dropdown = summary.parentElement;
			if (dropdown) {
				dropdown.open = true;
			}
		});
	});
});
