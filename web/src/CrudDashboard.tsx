import CssBaseline from '@mui/material/CssBaseline';
import { createHashRouter, RouterProvider, useNavigate } from 'react-router';
import DashboardLayout from './components/DashboardLayout';
import UsuarioList from './components/UsuarioList';
import UsuarioCrear from './components/UsuarioCrear';
import NotificationsProvider from './hooks/useNotifications/NotificationsProvider';
import DialogsProvider from './hooks/useDialogs/DialogsProvider';
import AppTheme from './theme/AppTheme';
import {
  dataGridCustomizations,
  datePickersCustomizations,
  sidebarCustomizations,
  formInputCustomizations,
} from './theme/customizations';
import React from 'react';
import { configurarUI } from './net/interceptor';
import useNotifications from './hooks/useNotifications/useNotifications';
import { useDialogs } from './hooks/useDialogs/useDialogs';
import UsuarioEdit from './components/UsuarioEdit';
import UsuarioPassword from './components/UsuarioPassword';

// Permite configurar el interceptor de axios
// con los componentes del UI
export const Main = () => {
  const dialogo = useDialogs();
  const notifica = useNotifications();
  const navigate = useNavigate();

  React.useEffect(() => {
    configurarUI(dialogo, notifica, () => navigate('/login'));
  }, [notifica, navigate]);

  return <DashboardLayout />;
};

const router = createHashRouter([
  {
    Component: Main,
    children: [
      {
        path: '/usuarios',
        Component: UsuarioList,
      },
      {
        path: '/usuarios/nuevo',
        Component: UsuarioCrear,
      },
      {
        path: '/usuarios/:id',
        Component: UsuarioEdit,
      },
      {
        path: '/usuarios/:id/password',
        Component: UsuarioPassword,
      },
      // Fallback route for the example routes in dashboard sidebar items
      {
        path: '*',
        Component: UsuarioList,
      },
    ],
  },
]);

const themeComponents = {
  ...dataGridCustomizations,
  ...datePickersCustomizations,
  ...sidebarCustomizations,
  ...formInputCustomizations,
};

export default function CrudDashboard(props: { disableCustomTheme?: boolean }) {
  return (
    <AppTheme {...props} themeComponents={themeComponents}>
      <CssBaseline enableColorScheme />
      <NotificationsProvider>
        <DialogsProvider>
          <RouterProvider router={router} />
        </DialogsProvider>
      </NotificationsProvider>
    </AppTheme>
  );
}