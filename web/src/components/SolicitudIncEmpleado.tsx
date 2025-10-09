import PageContainer from './PageContainer';
import useUsuarioLogeado from '../hooks/useUsuarioLogeado/useUsuarioLogeado';
import SolicitudIncidencia from './SolicitudIncidencia';
import Box from '@mui/material/Box';
import { FULL_HEIGHT_WIDTH } from '../context/DashboardSidebarContext';


export default function SolicitudIncEmpleado() {
  const { getUsrLogeado } = useUsuarioLogeado()
  const usuario = getUsrLogeado()

  return (
    <PageContainer title={'Solicitud incidencia empleado'}>
      <Box sx={{ ...FULL_HEIGHT_WIDTH }}>
        <SolicitudIncidencia
          usuarioId={usuario.id}
          solicitudEliminacion={false}
        />
      </Box>
    </PageContainer >
  );
}
