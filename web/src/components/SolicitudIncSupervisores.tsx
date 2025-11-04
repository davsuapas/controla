import PageContainer from './PageContainer';
import useUsuarioLogeado from '../hooks/useUsuarioLogeado/useUsuarioLogeado';
import SolicitudIncidencia from './SolicitudIncidencia';
import Box from '@mui/material/Box';
import { FULL_HEIGHT_WIDTH } from '../context/DashboardSidebarContext';
import Stack from '@mui/material/Stack';
import SelectorEmpleado from './SelectorEmpleado';
import React from 'react';
import { DescriptorUsuario, RolID } from '../modelos/usuarios';


export default function SolicitudIncSupervisores() {
  const { getUsrLogeado } = useUsuarioLogeado()
  const usuario = getUsrLogeado()
  const [empleado, setEmpleado] = React.useState<number | undefined>(undefined);
  const [isLoading, setIsLoading] = React.useState<boolean>(false);

  // Por defecto, solo puede hacer solicitudes de incidencias
  // de los marcajes realizados por el registrador
  let usuarioReg: number | undefined = usuario.id;

  if (usuario.tieneRol(RolID.Supervisor)) {
    // Si el usuario es el supervisor puede realizar todos
    // los tipos de solicitud para cualquier empleado
    // incluso aunque no registrará el el marcaje
    usuarioReg = undefined;
  }

  const handleEmpleadoChange = React.useCallback(
    (empleado: DescriptorUsuario | undefined) => {
      setEmpleado(empleado?.id)
    },
    []
  );

  return (
    <PageContainer title={'Solicitud incidencia para registradores o supervisores'}>
      <Box sx={{ mt: 3, ...FULL_HEIGHT_WIDTH }}>
        <Stack spacing={2} sx={{ height: '100%' }}>
          <SelectorEmpleado
            onChange={handleEmpleadoChange}
            disabled={false}
            onLoadingChange={setIsLoading}
          />
          <SolicitudIncidencia
            usuarioId={empleado}
            solicitudEliminacion={true}
            usuarioRegId={usuarioReg}
            isLoading={isLoading}
          />
        </Stack>
      </Box>
    </PageContainer >
  );
}
