import Box from '@mui/material/Box';
import PageContainer from './PageContainer';
import ResumenMarcaje from './ResumenMarcaje';
import FormGroup from '@mui/material/FormGroup';
import Grid from '@mui/material/Grid';
import { useState } from 'react';
import { DescriptorUsuario, RolID } from '../modelos/usuarios';
import useNotifications from '../hooks/useNotifications/useNotifications';
import React from 'react';
import { NetErrorControlado } from '../net/interceptor';
import InputLabel from '@mui/material/InputLabel';
import Select, { SelectChangeEvent } from '@mui/material/Select';
import MenuItem from '@mui/material/MenuItem';
import { api } from '../api/fabrica';
import { FULL_HEIGHT_WIDTH } from '../context/DashboardSidebarContext';
import { TimeField } from '@mui/x-date-pickers/TimeField';
import dayjs from 'dayjs';
import { DatePicker } from '@mui/x-date-pickers/DatePicker';
import Divider from '@mui/material/Divider';
import LocalizationProviderES from '../theme/location';
import Button from '@mui/material/Button';
import { MarcajeOutDTO } from '../modelos/dto';
import useUsuarioLogeado from '../hooks/useUsuarioLogeado/useUsuarioLogeado';
import { logError } from '../error';

interface FormularioData {
  empleado: DescriptorUsuario;
  fecha: dayjs.Dayjs;
  entrada: dayjs.Dayjs;
  salida: dayjs.Dayjs;
}

interface FormErrors {
  fecha: string; // No se usa pero mejora la legibilidad del código
  entrada: string;
  salida: string;
}

const HORA_NO_VALIDA = 'Hora no valida';

function validarFechaHora(fechaHora: dayjs.Dayjs | null | undefined) {
  return fechaHora && fechaHora.isValid()
}

// Permite realizar un marcaje para un usuario seleccionado 
// indicando manualmente la fecha, hora de inicio y fín.
// Además muestra el horario más cercano y los últimos
// marcajes del usuario.
export default function MarcajeManual() {
  const [empleados, setEmpleados] = useState<DescriptorUsuario[]>([]);

  const [formData, setFormData] = useState<Partial<FormularioData>>({});
  const [formErrors, setFormErrors] = useState<Partial<FormErrors>>({});

  const [isLoading, setIsLoading] = useState<boolean>(true);

  // Permite forzar el renderizado de los marcajes de resumen
  const [refreshTrigger, setRefreshTrigger] = useState(0);


  const usuarioLogeado = useUsuarioLogeado();
  const notifica = useNotifications();


  // Carga los últimos marcajes y el horario más cercano
  const cargarDatos = React.useCallback(async () => {
    setIsLoading(true);

    try {
      let empls = await api().usuarios.usuarios_por_rol(RolID.Empleado)
      setEmpleados(empls);

      setFormData({
        empleado: empls[0],
        fecha: dayjs(),
        entrada: undefined,
        salida: undefined,
      });
    } catch (error) {
      if (!(error instanceof NetErrorControlado)) {
        logError('marcajemanual.cargardatos', error);
        notifica.show(
          'Error inesperado al cargar los usuarios',
          {
            severity: 'error',
            autoHideDuration: 5000,
          },
        );
      }
      setEmpleados([]);
    }
    setIsLoading(false);
  }, []);

  const resetCamposHora = React.useCallback(() => {
    setFormData(prev => ({
      ...prev,
      entrada: undefined,
      salida: undefined,
    }));

    setFormErrors({});
  },
    []
  )

  React.useEffect(() => {
    cargarDatos();
  }, []);


  const handleEmpleadoChange = React.useCallback(
    (event: SelectChangeEvent<string>) => {
      const id = Number(event.target.value);
      const empleadoSeleccionado = empleados.find(u => u.id === id);

      setFormData({
        empleado: empleadoSeleccionado,
        fecha: dayjs(),
        entrada: undefined,
        salida: undefined,
      });
    }, [empleados]);


  // Maneja el cambio de los campos de fecha y hora
  const handleDateTimeFieldChange = React.useCallback(
    (name: string, value: dayjs.Dayjs | null) => {
      const valida = validarFechaHora(value);

      setFormErrors(prev => ({
        ...prev,
        [name]: valida ? undefined : HORA_NO_VALIDA
      }));

      setFormData(prev => ({
        ...prev,
        [name]: value
      }));
    }, []);

  const handleSubmit = React.useCallback(
    async (event: React.FormEvent<HTMLFormElement>) => {
      event.preventDefault();

      const validaFecha = validarFechaHora(formData.fecha);
      const validaEntrada = validarFechaHora(formData.entrada);
      let validaSalida = validarFechaHora(formData.salida);

      let msg_salida_error = HORA_NO_VALIDA;

      if (!(validaEntrada && validaSalida &&
        formData.entrada! < formData.salida!)) {
        validaSalida = false;
        msg_salida_error =
          'La hora de salida no puede ser menor que la de hora de entrada'
      }

      setFormErrors(
        {
          entrada: validaEntrada ? undefined : HORA_NO_VALIDA,
          salida: validaSalida ? undefined : msg_salida_error
        }
      );

      if (validaFecha &&
        validaEntrada &&
        validaSalida &&
        formData.empleado) {
        setIsLoading(true);

        try {
          await api().marcajes.registrar(
            MarcajeOutDTO.new(
              formData.empleado.id,
              usuarioLogeado.getUsrLogeado().toDescriptor(),
              formData.fecha!,
              formData.entrada!,
              formData.salida
            )
          )

          notifica.show('Marcaje registrado satisfactóriamente.', {
            severity: 'success',
            autoHideDuration: 5000,
          });

          resetCamposHora();
          // Incrementar el trigger para forzar recarga
          setRefreshTrigger(prev => prev + 1);
        } catch (error) {
          if (!(error instanceof NetErrorControlado)) {
            logError('marcajemanual.registrar', error);

            notifica.show(
              'Error inesperado al registrar el marcaje',
              {
                severity: 'error',
                autoHideDuration: 5000,
              },
            );
          }
        }

        setIsLoading(false);
      } else {
        notifica.show(
          'Imposible realizar el registro. Corriga los errores',
          {
            severity: 'warning',
            autoHideDuration: 5000,
          },
        );
      }

    },
    [formData],
  );

  const pageTitle = 'Marcaje manual del empleado';

  return (
    <PageContainer title={pageTitle}>
      <Box
        component="form"
        onSubmit={handleSubmit}
        noValidate
        autoComplete="off"
        sx={FULL_HEIGHT_WIDTH}
      >
        <LocalizationProviderES>
          <FormGroup>
            <Grid container spacing={2} sx={{ ml: 0.2, mb: 2, width: '100%' }}>
              <Grid size={{ xs: 12, sm: 12, md: 6 }}
                sx={{ display: 'flex', flexDirection: 'column' }}>
                <InputLabel>Empleado</InputLabel>
                <Select
                  name='empleado'
                  value={formData.empleado?.id?.toString() ?? ''}
                  label="Empleado"
                  onChange={handleEmpleadoChange}
                  disabled={isLoading}
                  fullWidth
                >
                  {empleados.map((empleado) => (
                    <MenuItem key={empleado.id} value={empleado.id.toString()}>
                      {empleado.nombreCompleto()}
                    </MenuItem>
                  ))}
                </Select>
              </Grid>
              <Grid size={{ xs: 12 }}>
                <Divider sx={{ my: 1, width: '100%' }} />
              </Grid>
              <Grid container spacing={2} sx={{ width: '100%' }}>
                <Grid size={{ xs: 12, sm: 12, md: 2 }}>
                  <DatePicker
                    name='fecha'
                    label="Fecha"
                    value={formData.fecha || null}
                    onChange={value =>
                      handleDateTimeFieldChange('fecha', value)}
                    sx={{ width: '100%' }}
                  />
                </Grid>
                <Grid size={{ xs: 12, sm: 12, md: 2 }}>
                  <TimeField
                    name='entrada'
                    label="Hora de entrada"
                    value={formData.entrada || null}
                    onChange={value =>
                      handleDateTimeFieldChange('entrada', value)}
                    error={!!formErrors.entrada}
                    helperText={formErrors.entrada ?? ' '}
                    sx={{ width: '100%' }}
                  />
                </Grid>
                <Grid size={{ xs: 12, sm: 12, md: 2 }}>
                  <TimeField
                    name='salida'
                    label="Hora de salida"
                    value={formData.salida || null}
                    onChange={value =>
                      handleDateTimeFieldChange('salida', value)}
                    error={!!formErrors.salida}
                    helperText={formErrors.salida ?? ' '}
                    sx={{ width: '100%' }}
                  />
                </Grid>
                <Grid size={{ xs: 12, sm: 12, md: 4 }}>
                  <Button
                    type="submit"
                    variant="contained"
                    size="medium"
                    disabled={isLoading}
                    sx={{ width: '100%', m: 1 }}
                  >
                    REGISTRAR
                  </Button>
                </Grid>
              </Grid>
            </Grid>
          </FormGroup>
        </LocalizationProviderES>
        <ResumenMarcaje
          ultimosMarcajes={false}
          usuarioId={formData.empleado?.id.toString()}
          fecha={formData.fecha}
          horaInicio={formData.entrada}
          refreshTrigger={refreshTrigger} />
      </Box>
    </PageContainer >
  );
}
