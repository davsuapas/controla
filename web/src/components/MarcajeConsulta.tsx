import Box from '@mui/material/Box';
import MarcajeList from './MarcajeList';
import { Marcaje } from '../modelos/marcaje';
import React, { useState } from 'react';
import { NetErrorControlado } from '../net/interceptor';
import useNotifications from '../hooks/useNotifications/useNotifications';
import { logError } from '../error';
import PageContainer from './PageContainer';
import { FULL_HEIGHT_WIDTH } from '../context/DashboardSidebarContext';
import { api } from '../api/fabrica';
import { Backdrop, Button, CircularProgress, Grid } from '@mui/material';
import { SelectorFechas, SelectorFechasRef } from './SelectorFechas';
import useUsuarioLogeado from '../hooks/useUsuarioLogeado/useUsuarioLogeado';
import { DescriptorUsuario, filtroUsuarioRegistra, RolID } from '../modelos/usuarios';
import SelectorEmpleado from './SelectorEmpleado';

// Muestra los marcajes según un filtro
// Esta pantalla solo puede ser usada por empleados, registrador o supervisor
export default function ConsultaMarcaje() {
  const selectorFechasRef = React.useRef<SelectorFechasRef>(null);

  const [marcaje, setMarcaje] = useState<Marcaje[]>([]);
  const [isLoading, setIsLoading] = useState<boolean>(true);
  const [bloquear, setBloquear] = React.useState<boolean>(false);

  const usuarioLog = useUsuarioLogeado().getUsrLogeado();
  const notifica = useNotifications();

  const usuarioSoloEmpleado =
    !usuarioLog.tieneRol(RolID.Registrador) &&
    !usuarioLog.tieneRol(RolID.Supervisor);

  const [empleado, setEmpleado] = React.useState<number>(usuarioLog.id);

  // Carga los últimos marcajes (solo depende de usuarioId)
  const cargarMarcaje = React.useCallback(
    async () => {
      setIsLoading(true);

      if (!selectorFechasRef.current) {
        console.log('filtrosRef.current es vacío en MarcajeConsulta');
        return;
      }

      const { fechaInicio, fechaFin } = selectorFechasRef.current.getFormData();

      try {
        let marcajesData: Marcaje[] =
          await api().marcajes.marcajes(
            empleado,
            fechaInicio,
            fechaFin,
            filtroUsuarioRegistra(empleado, usuarioLog) ?? null
          );
        setMarcaje(marcajesData);
      } catch (error) {
        if (!(error instanceof NetErrorControlado)) {
          logError('consulta-marcaje.cargar.marcajes', error);
          notifica.show(
            'Error inesperado al cargar los marcajes',
            {
              severity: 'error',
              autoHideDuration: 5000,
            },
          );
        }
        setMarcaje([]);
      }

      setIsLoading(false);
    }, [empleado]);

  React.useEffect(() => {
    cargarMarcaje();
  }, [empleado]);

  const handleEmpleadoChange = React.useCallback(
    (empleado: DescriptorUsuario) => {
      setEmpleado(empleado.id);
    },
    []
  );

  const handleFiltrar = React.useCallback(async () => {
    cargarMarcaje()
  }, [cargarMarcaje]);

  return (
    <PageContainer title={'Marcajes registrados'}>
      <Box sx={{ ...FULL_HEIGHT_WIDTH }}>
        {!usuarioSoloEmpleado && (
          <>
            <SelectorEmpleado
              onChange={handleEmpleadoChange}
              onLoadingChange={setBloquear}
              usuarioPorDefecto={usuarioLog.id}
            />
            <Box sx={{ mb: 3 }} />
          </>
        )}
        <Grid container spacing={2}
          sx={{ mt: 2, ml: 0.2, mb: 2, width: '100%' }}>
          <SelectorFechas
            ref={selectorFechasRef}
            labelUltimosRegistros={'Últimos marcajes'} />
          <Grid size={{ xs: 12, sm: 12, md: 1 }}>
            <Button
              variant="contained"
              sx={{
                width: { xs: '100%', sm: 'auto' },
                minWidth: 120,
                mt: 0.5
              }}
              disabled={isLoading || bloquear}
              onClick={handleFiltrar}
            >
              FILTRAR
            </Button>
          </Grid>
        </Grid>
        <Box sx={{ flex: 1, overflow: 'auto', position: 'relative' }}>
          <Backdrop
            sx={{
              zIndex: (theme) => theme.zIndex.drawer + 1,
              position: 'absolute'
            }}
            open={isLoading}
          >
            <CircularProgress color="inherit" />
          </Backdrop>
          <MarcajeList marcajes={marcaje} />
        </Box>
      </Box>
    </PageContainer>
  );
}