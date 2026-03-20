import * as React from 'react';
import { useLocation, useNavigate } from 'react-router';
import useNotifications from '../hooks/useNotifications/useNotifications';
import HorarioForm, {
  HorarioFormState,
  HorarioFormValues,
  validaHorario,
  FormFieldValue
} from './HorarioForm';
import PageContainer from './PageContainer';
import dayjs from 'dayjs';
import { NetErrorControlado } from '../net/interceptor';
import { api } from '../api/fabrica';
import { logError } from '../error';
import { ConfigHorario, DiaSemana, Horario } from '../modelos/usuarios';

// Mapeo de dayjs().day() (0=Domingo, 1=Lunes...) a DiaSemana
const getDiaActual = (): DiaSemana => {
  const dayIndex = dayjs().day();
  const map = [
    DiaSemana.Domingo,   // 0
    DiaSemana.Lunes,     // 1
    DiaSemana.Martes,    // 2
    DiaSemana.Miércoles, // 3
    DiaSemana.Jueves,    // 4
    DiaSemana.Viernes,   // 5
    DiaSemana.Sábado     // 6
  ];
  return map[dayIndex];
};

const INITIAL_FORM_VALUES: HorarioFormValues = {
  dia: getDiaActual(),
  horas: 8,
  cortesia: 0,
  caducidadFechaIni: null,
  caducidadFechaFin: null,
};

export default function HorarioCrear() {
  const navigate = useNavigate();
  const location = useLocation();
  const notifica = useNotifications();

  const [formState, setFormState] = React.useState<HorarioFormState>(() => ({
    values: INITIAL_FORM_VALUES,
    errors: {},
  }));

  const setFormErrors = React.useCallback(
    (newFormErrors: Partial<HorarioFormState['errors']>) => {
      setFormState((previousState) => ({
        ...previousState,
        errors: newFormErrors,
      }));
    },
    [],
  );

  const handleFormFieldChange = React.useCallback(
    (name: keyof HorarioFormValues, value: FormFieldValue) => {
      setFormState((currentState) => {
        const newFormValues = {
          ...currentState.values,
          [name]: value
        };

        const validateAndUpdateErrors = () => {
          const { issues } = validaHorario(newFormValues);
          const fieldError = issues?.find((issue) => issue.path?.[0] === name)?.message;

          setFormState((prevState) => ({
            ...prevState,
            errors: {
              ...prevState.errors,
              [name]: fieldError,
            }
          }));
        };

        validateAndUpdateErrors();

        return {
          ...currentState,
          values: newFormValues,
        };
      });
    },
    [],
  );

  const handleFormReset = React.useCallback(() => {
    setFormState({
      values: INITIAL_FORM_VALUES,
      errors: {}
    });
  }, []);

  const handleFormSubmit = React.useCallback(async (values: HorarioFormValues) => {
    const { issues } = validaHorario(values);

    if (issues && issues.length > 0) {
      setFormErrors(
        Object.fromEntries(
          issues.map((issue) => [issue.path?.[0], issue.message])),
      );
      notifica.show('Imposible crear el horario. Corrija los errores',
        { severity: 'warning', autoHideDuration: 5000 });
      return;
    }

    try {
      const state = location.state as { usuarioId: number, fechaCreacion: dayjs.Dayjs } | null;
      if (!state?.usuarioId || !state?.fechaCreacion) {
        notifica.show('Faltan datos del usuario o fecha de configuración',
          { severity: 'error', autoHideDuration: 5000 });
        return;
      }

      const nuevoHorario = new ConfigHorario({
        id: 0,
        usuario: state.usuarioId,
        fechaCreacion: state.fechaCreacion,
        caducidadFechaIni: values.caducidadFechaIni,
        caducidadFechaFin: values.caducidadFechaFin,
        cortesia: values.cortesia,
        horario: new Horario({
          id: 0,
          dia: values.dia as DiaSemana,
          horas: values.horas!,
        })
      });

      await api().usuarios.crearHorario(nuevoHorario);

      notifica.show('Horario creado satisfactoriamente',
        { severity: 'success', autoHideDuration: 5000 });

      navigate('/horarios', {
        state: {
          usuarioId: state?.usuarioId
        }
      });

    } catch (error) {
      if (error instanceof NetErrorControlado) return;
      logError('horario-crear.crear', error);
      notifica.show('Error inesperado al crear el horario',
        { severity: 'error', autoHideDuration: 5000 });
    }
  }, [notifica, navigate, setFormErrors, location.state]);

  const handleBack = React.useCallback(() => {
    const state = location.state as { usuarioId: number, fechaCreacion: dayjs.Dayjs } | null;
    navigate('/horarios', {
      state: {
        usuarioId: state?.usuarioId
      }
    });
  }, [navigate, location.state]);

  return (
    <PageContainer title="Nuevo horario" breadcrumbs={[{ title: 'Horarios', path: '/horarios' }, { title: 'Nuevo' }]}>
      <HorarioForm
        formState={formState}
        onFieldChange={handleFormFieldChange}
        onSubmit={handleFormSubmit}
        onReset={handleFormReset}
        onBack={handleBack}
        submitButtonLabel="CREAR"
      />
    </PageContainer>
  );
}