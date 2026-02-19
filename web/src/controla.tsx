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
import SolicitudIncidencia from './components/IncidenciaSolicitud';
import ErrorPage from './components/ErrorPage';
import ErrorBoundary from './components/ErrorBoundary';
import RevisionIncidencia from './components/IncidenciaRevision';
import GestionIncidencia from './components/IncidenciaGestion';
import MarcajeAuto from './components/MarcajeAuto';
import MarcajeConsulta from './components/MarcajeConsulta';
import { URL_BASE_ROUTER } from './config';
import HorarioConfigurador from './components/HorarioConfigurador';
import HorarioCrear from './components/HorarioCrear';
import HorarioEdit from './components/HorarioEdit';
import CalendarioList from './components/CalendarioList';
import CalendarioCrear from './components/CalendarioCrear';
import CalendarioEdit from './components/CalendarioEdit';
import CalendarioFechas from './components/CalendarioFechas';
import CalendarioFechaCrear from './components/CalendarioFechaCrear';
import CalendarioFechaEdit from './components/CalendarioFechaEdit';
import InformeCumplimientoHorario from './components/InformeCumplimientoHorario';

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
        to='/'
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
            path: 'horarios',
            errorElement: <ErrorPage />,
            children: [
              {
                index: true,
                Component: HorarioConfigurador,
                errorElement: <ErrorPage />,
              },
              {
                path: 'nuevo',
                Component: HorarioCrear,
                errorElement: <ErrorPage />,
              },
              {
                path: ':id',
                Component: HorarioEdit,
                errorElement: <ErrorPage />,
              },
            ]
          },
          {
            path: 'calendarios',
            errorElement: <ErrorPage />,
            children: [
              {
                index: true,
                Component: CalendarioList,
                errorElement: <ErrorPage />,
              },
              {
                path: 'nuevo',
                Component: CalendarioCrear,
                errorElement: <ErrorPage />,
              },
              {
                path: ':id',
                Component: CalendarioEdit,
                errorElement: <ErrorPage />,
              },
              {
                path: ':id/fechas',
                Component: CalendarioFechas,
                errorElement: <ErrorPage />,
              },
              {
                path: ':id/fechas/nuevo',
                Component: CalendarioFechaCrear,
                errorElement: <ErrorPage />,
              },
              {
                path: ':id/fechas/:fechaId',
                Component: CalendarioFechaEdit,
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
              {
                path: 'auto',
                Component: MarcajeAuto,
                errorElement: <ErrorPage />,
              },
              {
                path: 'consulta',
                Component: MarcajeConsulta,
                errorElement: <ErrorPage />,
              }
            ]
          },
          {
            path: 'incidencias',
            errorElement: <ErrorPage />,
            children: [
              {
                path: 'solicitud',
                Component: SolicitudIncidencia,
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
            ]
          },
          {
            path: 'informes',
            errorElement: <ErrorPage />,
            children: [
              {
                path: 'cumplimiento',
                Component: InformeCumplimientoHorario,
                errorElement: <ErrorPage />,
              },
            ]
          },
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

const router = createBrowserRouter(rutas, { basename: URL_BASE_ROUTER });

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