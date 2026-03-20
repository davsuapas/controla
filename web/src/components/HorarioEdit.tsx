import * as React from 'react';
import { useNavigate, useParams } from 'react-router';
import useNotifications from '../hooks/useNotifications/useNotifications';
import HorarioForm, {
  HorarioFormState,
  HorarioFormValues,
  validaHorario,
  FormFieldValue
} from './HorarioForm';
import PageContainer from './PageContainer';
import { NetErrorControlado } from '../net/interceptor';
import { api } from '../api/fabrica';
import { logError } from '../error';
import { useIsMounted } from '../hooks/useComponentMounted';
import Box from '@mui/material/Box';
import CircularProgress from '@mui/material/CircularProgress';
import Alert from '@mui/material/Alert';
import dayjs from 'dayjs';
import { ConfigHorario, DiaSemana, Horario } from '../modelos/usuarios';

export default function HorarioEdit() {
  const { id } = useParams();
  const navigate = useNavigate();
  const notifica = useNotifications();
  const isMounted = useIsMounted();

  const [isLoading, setIsLoading] = React.useState(true);
  const [error, setError] = React.useState<Error | null>(null);
  const [initialValues, setInitialValues] = React.useState<HorarioFormValues | null>(null);
  const [configHorario, setConfigHorario] = React.useState<ConfigHorario | null>(null);

  const [formState, setFormState] = React.useState<HorarioFormState>({
    values: {
      dia: DiaSemana.Lunes,
      horas: 8,
      cortesia: 0,
      caducidadFechaIni: null,
      caducidadFechaFin: null
    },
    errors: {},
  });

  const loadData = React.useCallback(async () => {
    setError(null);
    setIsLoading(true);

    try {
      const data = await api().usuarios.horario(Number(id));

      const loadedValues: HorarioFormValues = {
        dia: data.horario.dia as DiaSemana,
        horas: data.horario.horas,
        cortesia: data.cortesia,
        caducidadFechaIni: data.caducidadFechaIni ?
          dayjs(data.caducidadFechaIni) : null,
        caducidadFechaFin: data.caducidadFechaFin ?
          dayjs(data.caducidadFechaFin) : null
      };

      if (isMounted.current) {
        setConfigHorario(data);
        setInitialValues(loadedValues);
        setFormState(prev => ({ ...prev, values: loadedValues }));
      }
    } catch (error) {
      if (!(error instanceof NetErrorControlado)) {
        logError('horario-edit.cargar', error);
        if (isMounted.current) {
          setError(Error('Error inesperado al cargar el horario'));
        }
      }
    } finally {
      if (isMounted.current) {
        setIsLoading(false);
      }
    }
  }, [id, isMounted]);

  React.useEffect(() => {
    loadData();
  }, [loadData]);

  const handleFormFieldChange = React.useCallback(
    (name: keyof HorarioFormValues, value: FormFieldValue) => {
      setFormState((currentState) => {
        const newFormValues = {
          ...currentState.values,
          [name]: value
        };

        const { issues } = validaHorario(newFormValues);
        const fieldError = issues?.find((issue) => issue.path?.[0] === name)?.message;

        return {
          values: newFormValues,
          errors: {
            ...currentState.errors,
            [name]: fieldError,
          }
        };
      });
    },
    []
  );

  const handleFormReset = React.useCallback(() => {
    if (initialValues) {
      setFormState({ values: initialValues, errors: {} });
    }
  }, [initialValues]);

  const handleFormSubmit = React.useCallback(async (values: HorarioFormValues) => {
    const { issues } = validaHorario(values);

    if (issues && issues.length > 0) {
      setFormState(prev => ({
        ...prev,
        errors: Object.fromEntries(issues.map((issue) => [issue.path?.[0], issue.message]))
      }));
      notifica.show('Imposible actualizar el horario. Corrija los errores',
        { severity: 'warning', autoHideDuration: 5000, });
      return;
    }

    try {
      if (!configHorario) return;

      const horarioActualizado = new ConfigHorario({
        id: configHorario.id,
        usuario: configHorario.usuario,
        fechaCreacion: configHorario.fechaCreacion,
        caducidadFechaIni: values.caducidadFechaIni,
        caducidadFechaFin: values.caducidadFechaFin,
        cortesia: values.cortesia,
        horario: new Horario({
          id: configHorario.horario.id,
          dia: values.dia,
          horas: values.horas!,
        })
      });

      await api().usuarios.actualizarHorario(horarioActualizado);

      notifica.show('Horario actualizado satisfactoriamente',
        { severity: 'success', autoHideDuration: 5000 });

      navigate('/horarios', {
        state: {
          usuarioId: configHorario.usuario
        }
      });
    } catch (error) {
      if (error instanceof NetErrorControlado) return;
      logError('horario-edit.actualizar', error);
      notifica.show('Error inesperado al actualizar el horario',
        { severity: 'error', autoHideDuration: 5000 });
    }
  }, [id, notifica, navigate, configHorario]);

  const handleBack = React.useCallback(() => {
    navigate('/horarios', {
      state: {
        usuarioId: configHorario?.usuario
      }
    });
  }, [navigate, configHorario]);

  if (isLoading) {
    return (
      <Box sx={{ display: 'flex', justifyContent: 'center', mt: 4 }}>
        <CircularProgress />
      </Box>
    );
  }

  if (error) {
    return <Alert severity="error">{error.message}</Alert>;
  }

  return (
    <PageContainer
      title={`Edición de horario: ${id}`}
      breadcrumbs={[{ title: 'Horarios', path: '/horarios' }, { title: 'Edición' }]}>
      <HorarioForm
        formState={formState}
        onFieldChange={handleFormFieldChange}
        onSubmit={handleFormSubmit}
        onReset={handleFormReset}
        submitButtonLabel="ACTUALIZAR"
        onBack={handleBack}
      />
    </PageContainer>
  );
}