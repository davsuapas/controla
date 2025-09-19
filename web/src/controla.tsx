import CssBaseline from '@mui/material/CssBaseline';
import { createBrowserRouter, Navigate, Outlet, RouterProvider, useLocation } from 'react-router';
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
import { configurarInterceptor } from './net/interceptor';
import useNotifications from './hooks/useNotifications/useNotifications';
import { useDialogs } from './hooks/useDialogs/useDialogs';
import UsuarioEdit from './components/UsuarioEdit';
import UsuarioPassword from './components/UsuarioPassword';
import Login from './components/Login';
import UsuarioLogeadoProvider from './hooks/useUsuarioLogeado/UsuarioLogeadoProvider';
import useUsuarioLogeado from './hooks/useUsuarioLogeado/useUsuarioLogeado';
import Logout from './components/Logout';
import { crearAPI } from './api/usuarios';
import UsuarioShow from './components/UsuarioShow';


crearAPI(true);


// Layout raíz que permite usar los hooks
const RootLayout = () => {
  return (
    <NotificationsProvider>
      <DialogsProvider>
        <Outlet />
      </DialogsProvider>
    </NotificationsProvider>
  );
};

// Componente de protección de rutas
const ProtectedRoute = ({ children }: { children: React.ReactNode }) => {
  const { hayUsrLogeado } = useUsuarioLogeado();
  const location = useLocation();

  if (!hayUsrLogeado()) {
    return (
      <Navigate
        to="/"
        state={{ redirect: location.pathname + location.search }}
        replace
      />
    );
  }

  return <>{children}</>;
};

// Dashboard simplificado - solo maneja el interceptor
export const Dashboard = () => {
  const dialogo = useDialogs();
  const notifica = useNotifications();
  const usrLogeado = useUsuarioLogeado();

  React.useEffect(() => {
    configurarInterceptor(dialogo, notifica, usrLogeado);
  }, []);

  return <DashboardLayout />;
};

// Componente wrapper que combina protección + dashboard
const ProtectedDashboard = () => {
  return (
    <ProtectedRoute>
      <Dashboard />
    </ProtectedRoute>
  );
};

const rutas = [
  {
    path: '/',
    Component: RootLayout,
    children: [
      {
        index: true,
        Component: Login,
      },
      {
        Component: ProtectedDashboard,
        children: [
          {
            path: 'usuarios',
            children: [
              {
                index: true,
                Component: UsuarioList,
              },
              {
                path: 'nuevo',
                Component: UsuarioCrear,
              },
              {
                path: ':id',
                Component: UsuarioEdit,
              },
              {
                path: ':id/password',
                Component: UsuarioPassword,
              },
            ]
          },
          {
            path: 'miarea',
            children: [
              {
                path: 'password',
                Component: UsuarioPassword,
              },
              {
                path: 'perfil',
                Component: UsuarioShow,
              },
              {
                path: 'logout',
                Component: Logout,
              },
            ]
          }
        ],
      },
    ],
  },
];

const router = createBrowserRouter(rutas);

const themeComponents = {
  ...dataGridCustomizations,
  ...datePickersCustomizations,
  ...sidebarCustomizations,
  ...formInputCustomizations,
};

export default function Controla(props: { disableCustomTheme?: boolean }) {
  return (
    <AppTheme {...props} themeComponents={themeComponents}>
      <CssBaseline enableColorScheme />
      <UsuarioLogeadoProvider>
        <RouterProvider router={router} />
      </UsuarioLogeadoProvider>
    </AppTheme>
  );
}