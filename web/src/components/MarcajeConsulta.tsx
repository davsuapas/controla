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
import { Backdrop, Button, CircularProgress, Grid, useMediaQuery, useTheme } from '@mui/material';
import { SelectorFechas, SelectorFechasRef } from './SelectorFechas';
import useUsuarioLogeado from '../hooks/useUsuarioLogeado/useUsuarioLogeado';
import { DescriptorUsuario, filtroUsuarioRegistra, RolID } from '../modelos/usuarios';
import SelectorEmpleado from './SelectorEmpleado';
import ExpandMoreIcon from '@mui/icons-material/ExpandMore';
import ExpandLessIcon from '@mui/icons-material/ExpandLess';
import { useIsMounted } from '../hooks/useComponentMounted';

// Muestra los marcajes según un filtro
// Esta pantalla solo puede ser usada por empleados, registrador o supervisor
export default function ConsultaMarcaje() {
  const isMounted = useIsMounted();
  const theme = useTheme();
  const usuarioLog = useUsuarioLogeado().getUsrLogeado();
  const notifica = useNotifications();

  const selectorFechasRef = React.useRef<SelectorFechasRef>(null);

  const [marcaje, setMarcaje] = useState<Marcaje[]>([]);
  const [isLoading, setIsLoading] = useState<boolean>(true);
  const [bloquear, setBloquear] = React.useState<boolean>(false);

  const usuarioSoloEmpleado =
    !usuarioLog.tieneRol(RolID.Registrador) &&
    !usuarioLog.tieneRol(RolID.Supervisor);

  const [empleado, setEmpleado] = React.useState<number>(usuarioLog.id);

  const isMobile = useMediaQuery(theme.breakpoints.down('sm'));
  const [isExpanded, setIsExpanded] = React.useState<boolean>(false);

  // Carga los últimos marcajes (solo depende de usuarioId)
  const cargarMarcaje = React.useCallback(
    async () => {
      setIsLoading(true);

      if (!selectorFechasRef.current) {
        console.log('filtrosRef.current es vacío en MarcajeConsulta');
        return;
      }

      const { fechaInicio, fechaFin } = selectorFechasRef.current.getFormData();

      let marcajesData: Marcaje[] = []

      try {
        marcajesData =
          await api().marcajes.marcajes(
            empleado,
            fechaInicio,
            fechaFin,
            filtroUsuarioRegistra(empleado, usuarioLog) ?? null
          );
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
      }

      if (isMounted.current) {
        setMarcaje(marcajesData);
        setIsLoading(false);
      };
    }, [empleado, usuarioLog, notifica]);

  React.useEffect(() => {
    cargarMarcaje();
  }, [empleado, cargarMarcaje]);

  const handleEmpleadoChange = React.useCallback(
    (empleado: DescriptorUsuario) => {
      setEmpleado(empleado.id);
    },
    []
  );

  const handleFiltrar = React.useCallback(async () => {
    cargarMarcaje()
  }, [cargarMarcaje]);

  const toggleExpand = React.useCallback(() => {
    setIsExpanded(prev => !prev);
  }, []);

  return (
    <PageContainer title={'Marcajes registrados'}>
      <Box sx={{
        ...FULL_HEIGHT_WIDTH,
        display: 'flex',
        flexDirection: 'column'
      }}>
        {/* Contenido superior - Se oculta en móvil cuando está expandido */}
        <Box sx={{
          display: isMobile && isExpanded ? 'none' : 'block',
          flexShrink: 0
        }}>
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
        </Box>
        {/* Botón para expandir/contraer en móviles */}
        {isMobile && (
          <Box
            onClick={toggleExpand}
            sx={{
              height: 20,
              flexShrink: 0,
              display: 'flex',
              alignItems: 'center',
              justifyContent: 'center',
              cursor: 'pointer',
              borderBottom: '1px solid',
              borderColor: 'divider',
              bgcolor: 'background.paper',
              '&:active': {
                bgcolor: 'action.hover'
              },
              transition: 'background-color 0.2s'
            }}
          >
            {isExpanded ? (
              <ExpandMoreIcon sx={{ color: 'text.secondary' }} />
            ) : (
              <ExpandLessIcon sx={{ color: 'text.secondary' }} />
            )}
          </Box>
        )}
        {/* Contenedor de la tabla */}
        <Box sx={{
          flex: 1,
          minHeight: 250,
          position: 'relative',
        }}>
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