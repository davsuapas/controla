import CssBaseline from '@mui/material/CssBaseline';
import { createBrowserRouter, Navigate, Outlet, RouterProvider, useLocation } from 'react-router';
import DashboardLayout from './components/DashboardLayout';
import UsuarioList from './components/UsuarioList';
import UsuarioCrear from './components/UsuarioCrear';
import NotificationsProvider from './hooks/useNotifications/NotificationsProvider';
import DialogsProvider from './hooks/useDialogs/DialogsProvider';
import AppTheme from './theme/AppTheme';
import {
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
import UsuarioShow from './components/UsuarioShow';
import MarcajeManual from './components/MarcajeManual';
import { crearAPI } from './api/fabrica';
import SolicitudIncEmpleado from './components/SolicitudIncEmpleado';
import SolicitudIncSupervisores from './components/SolicitudIncSupervisores';
import ErrorPage from './components/ErrorPage';
import ErrorBoundary from './components/ErrorBoundary';
import RevisionIncidencia from './components/RevisionIncidencias';
import GestionIncidencia from './components/GestionIncidencia';

crearAPI(false);

// Layout raíz que permite usar los hooks
const RootLayout = () => {
  return (
    <ErrorBoundary>
      <NotificationsProvider>
        <DialogsProvider>
          <Outlet />
        </DialogsProvider>
      </NotificationsProvider>
    </ErrorBoundary>
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

// Componente wrapper que combina protección + dashboard
const GestionIncidenciaSuper = () => {
  return (
    <GestionIncidencia supervisor />
  );
};

const rutas = [
  {
    path: '/',
    Component: RootLayout,
    errorElement: <ErrorPage />, // Error boundary para el layout raíz
    children: [
      {
        index: true,
        Component: Login,
        errorElement: <ErrorPage />,
      },
      {
        Component: ProtectedDashboard,
        errorElement: <ErrorPage />,
        children: [
          {
            path: 'usuarios',
            errorElement: <ErrorPage />,
            children: [
              {
                index: true,
                Component: UsuarioList,
                errorElement: <ErrorPage />,
              },
              {
                path: 'nuevo',
                Component: UsuarioCrear,
                errorElement: <ErrorPage />,
              },
              {
                path: ':id',
                Component: UsuarioEdit,
                errorElement: <ErrorPage />,
              },
              {
                path: ':id/password',
                Component: UsuarioPassword,
                errorElement: <ErrorPage />,
              },
            ]
          },
          {
            path: 'miarea',
            errorElement: <ErrorPage />,
            children: [
              {
                path: 'password',
                Component: UsuarioPassword,
                errorElement: <ErrorPage />,
              },
              {
                path: 'perfil',
                Component: UsuarioShow,
                errorElement: <ErrorPage />,
              },
              {
                path: 'logout',
                Component: Logout,
                errorElement: <ErrorPage />,
              },
            ]
          },
          {
            path: 'marcaje',
            errorElement: <ErrorPage />,
            children: [
              {
                path: 'manual',
                Component: MarcajeManual,
                errorElement: <ErrorPage />,
              },
            ]
          },
          {
            path: 'incidencias',
            errorElement: <ErrorPage />,
            children: [
              {
                path: 'solicitud',
                Component: SolicitudIncEmpleado,
                errorElement: <ErrorPage />,
              },
              {
                path: 'solicitud/privilegios',
                Component: SolicitudIncSupervisores,
                errorElement: <ErrorPage />,
              },
              {
                path: 'revision',
                Component: RevisionIncidencia,
                errorElement: <ErrorPage />,
              },
              {
                path: 'gestion',
                Component: GestionIncidencia,
                errorElement: <ErrorPage />,
              },
              {
                path: 'gestion/supervisor',
                Component: GestionIncidenciaSuper,
                errorElement: <ErrorPage />,
              },
            ]
          }
        ],
      },
      // Ruta comodín para 404
      {
        path: '*',
        element: <ErrorPage type="404" />,
      }
    ],
  },
];

const router = createBrowserRouter(rutas);

const themeComponents = {
  ...sidebarCustomizations,
  ...formInputCustomizations,
};

export default function Controla(props: { disableCustomTheme?: boolean }) {
  return (
    <ErrorBoundary>
      <AppTheme {...props} themeComponents={themeComponents}>
        <CssBaseline enableColorScheme />
        <UsuarioLogeadoProvider>
          <RouterProvider router={router} />
        </UsuarioLogeadoProvider>
      </AppTheme>
    </ErrorBoundary>
  );
}