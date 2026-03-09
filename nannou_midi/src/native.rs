use std::time::{Duration, Instant};

use bevy::prelude::*;
use crossbeam_channel::{Receiver, Sender};

use crate::components::*;
use crate::events::*;

#[derive(Component)]
pub(crate) struct NativeInputPort(pub midir::MidiInputPort);

#[derive(Component)]
pub(crate) struct NativeOutputPort(pub midir::MidiOutputPort);

#[derive(Component)]
struct InputConnection {
    _connection: midir::MidiInputConnection<()>,
    receiver: Receiver<MidiData>,
}

#[derive(Component)]
struct OutputConnection {
    connection: midir::MidiOutputConnection,
}

#[derive(Resource)]
struct PortEnumerationTimer {
    last_check: Instant,
}

pub(crate) fn init(app: &mut App) {
    app.insert_resource(PortEnumerationTimer {
        last_check: Instant::now() - Duration::from_secs(10),
    });

    app.add_systems(
        PreUpdate,
        (
            enumerate_midi_ports,
            open_midi_inputs,
            open_midi_outputs,
            receive_midi_messages,
            send_midi_messages,
        )
            .chain(),
    );

    app.add_observer(on_midi_input_removed);
    app.add_observer(on_midi_output_removed);
}

fn enumerate_midi_ports(
    mut commands: Commands,
    existing_ports: Query<(Entity, &Name, &MidiPort)>,
    mut timer: ResMut<PortEnumerationTimer>,
) {
    let now = Instant::now();
    if now.duration_since(timer.last_check) < Duration::from_secs(2) {
        return;
    }
    timer.last_check = now;

    let input_ports = match midir::MidiInput::new("nannou_midi_enum") {
        Ok(midi_in) => {
            let ports = midi_in.ports();
            ports
                .into_iter()
                .filter_map(|p| midi_in.port_name(&p).ok().map(|name| (name, p)))
                .collect::<Vec<_>>()
        }
        Err(err) => {
            warn!("failed to enumerate MIDI input ports: {err}");
            Vec::new()
        }
    };

    let output_ports = match midir::MidiOutput::new("nannou_midi_enum") {
        Ok(midi_out) => {
            let ports = midi_out.ports();
            ports
                .into_iter()
                .filter_map(|p| midi_out.port_name(&p).ok().map(|name| (name, p)))
                .collect::<Vec<_>>()
        }
        Err(err) => {
            warn!("failed to enumerate MIDI output ports: {err}");
            Vec::new()
        }
    };

    let mut matched_entities = Vec::new();

    for (name, native_port) in input_ports {
        let already_exists = existing_ports.iter().any(|(_, n, p)| {
            n.as_str() == name && p.direction == MidiPortDirection::Input
        });
        if !already_exists {
            let entity = commands
                .spawn((
                    Name::new(name),
                    MidiPort {
                        direction: MidiPortDirection::Input,
                    },
                    NativeInputPort(native_port),
                ))
                .id();
            commands
                .entity(entity)
                .trigger(|e| MidiPortAdded { entity: e });
        } else {
            for (e, n, p) in &existing_ports {
                if n.as_str() == name && p.direction == MidiPortDirection::Input {
                    matched_entities.push(e);
                    break;
                }
            }
        }
    }

    for (name, native_port) in output_ports {
        let already_exists = existing_ports.iter().any(|(_, n, p)| {
            n.as_str() == name && p.direction == MidiPortDirection::Output
        });
        if !already_exists {
            let entity = commands
                .spawn((
                    Name::new(name),
                    MidiPort {
                        direction: MidiPortDirection::Output,
                    },
                    NativeOutputPort(native_port),
                ))
                .id();
            commands
                .entity(entity)
                .trigger(|e| MidiPortAdded { entity: e });
        } else {
            for (e, n, p) in &existing_ports {
                if n.as_str() == name && p.direction == MidiPortDirection::Output {
                    matched_entities.push(e);
                    break;
                }
            }
        }
    }

    for (entity, _, _) in &existing_ports {
        if !matched_entities.contains(&entity) {
            commands
                .entity(entity)
                .trigger(|e| MidiPortRemoved { entity: e });
            commands.entity(entity).despawn();
        }
    }
}

fn open_midi_inputs(
    mut commands: Commands,
    new_inputs: Query<(Entity, &MidiInput, Option<&Name>), Added<MidiInput>>,
    ports: Query<(Entity, &MidiPort, &NativeInputPort)>,
) {
    for (entity, midi_input, name) in &new_inputs {
        let connection_name = name
            .map(|n| n.to_string())
            .unwrap_or_else(|| format!("{entity}"));
        let port_entity = if let Some(port) = midi_input.port {
            port
        } else {
            match ports
                .iter()
                .find(|(_, p, _)| p.direction == MidiPortDirection::Input)
            {
                Some((e, _, _)) => e,
                None => {
                    let msg = "no MIDI input port available";
                    warn!("{msg}");
                    commands.entity(entity).insert(MidiError {
                        message: msg.to_string(),
                    });
                    commands.entity(entity).trigger(|e| MidiDisconnected {
                        entity: e,
                        reason: msg.to_string(),
                    });
                    continue;
                }
            }
        };

        let Ok((_, _, native_port)) = ports.get(port_entity) else {
            let msg = "referenced MIDI port entity not found";
            commands.entity(entity).insert(MidiError {
                message: msg.to_string(),
            });
            commands.entity(entity).trigger(|e| MidiDisconnected {
                entity: e,
                reason: msg.to_string(),
            });
            continue;
        };

        let midi_in = match midir::MidiInput::new(&connection_name) {
            Ok(m) => m,
            Err(err) => {
                let msg = format!("failed to create MIDI input: {err}");
                commands.entity(entity).insert(MidiError {
                    message: msg.clone(),
                });
                let reason = msg;
                commands
                    .entity(entity)
                    .trigger(|e| MidiDisconnected { entity: e, reason });
                continue;
            }
        };

        let (sender, receiver) = crossbeam_channel::unbounded::<MidiData>();
        match connect_input(midi_in, &native_port.0, &connection_name, sender) {
            Ok(connection) => {
                commands.entity(entity).insert((
                    InputConnection {
                        _connection: connection,
                        receiver: receiver.clone(),
                    },
                    MidiInputStream::new(),
                ));
                commands
                    .entity(entity)
                    .trigger(|e| MidiConnected { entity: e });
            }
            Err(msg) => {
                commands.entity(entity).insert(MidiError {
                    message: msg.clone(),
                });
                let reason = msg;
                commands
                    .entity(entity)
                    .trigger(|e| MidiDisconnected { entity: e, reason });
            }
        }
    }
}

fn connect_input(
    midi_in: midir::MidiInput,
    port: &midir::MidiInputPort,
    connection_name: &str,
    sender: Sender<MidiData>,
) -> Result<midir::MidiInputConnection<()>, String> {
    midi_in
        .connect(
            port,
            connection_name,
            move |stamp, message, _| {
                if message.len() >= 3 {
                    let _ = sender.send(MidiData {
                        stamp,
                        message: MidiMessage::from([message[0], message[1], message[2]]),
                    });
                }
            },
            (),
        )
        .map_err(|e| format!("failed to connect MIDI input: {e}"))
}

fn open_midi_outputs(
    mut commands: Commands,
    new_outputs: Query<(Entity, &MidiOutput, Option<&Name>), Added<MidiOutput>>,
    ports: Query<(Entity, &MidiPort, &NativeOutputPort)>,
) {
    for (entity, midi_output, name) in &new_outputs {
        let connection_name = name
            .map(|n| n.to_string())
            .unwrap_or_else(|| format!("{entity}"));
        let port_entity = if let Some(port) = midi_output.port {
            port
        } else {
            let msg = "no MIDI output port specified";
            warn!("{msg}");
            commands.entity(entity).insert(MidiError {
                message: msg.to_string(),
            });
            commands.entity(entity).trigger(|e| MidiDisconnected {
                entity: e,
                reason: msg.to_string(),
            });
            continue;
        };

        let Ok((_, _, native_port)) = ports.get(port_entity) else {
            let msg = "referenced MIDI port entity not found";
            commands.entity(entity).insert(MidiError {
                message: msg.to_string(),
            });
            commands.entity(entity).trigger(|e| MidiDisconnected {
                entity: e,
                reason: msg.to_string(),
            });
            continue;
        };

        let midi_out = match midir::MidiOutput::new(&connection_name) {
            Ok(m) => m,
            Err(err) => {
                let msg = format!("failed to create MIDI output: {err}");
                commands.entity(entity).insert(MidiError {
                    message: msg.clone(),
                });
                let reason = msg;
                commands
                    .entity(entity)
                    .trigger(|e| MidiDisconnected { entity: e, reason });
                continue;
            }
        };

        match midi_out.connect(&native_port.0, &connection_name) {
            Ok(connection) => {
                commands
                    .entity(entity)
                    .insert((OutputConnection { connection }, MidiOutputStream::new()));
                commands
                    .entity(entity)
                    .trigger(|e| MidiConnected { entity: e });
            }
            Err(err) => {
                let msg = format!("failed to connect MIDI output: {err}");
                commands.entity(entity).insert(MidiError {
                    message: msg.clone(),
                });
                let reason = msg;
                commands
                    .entity(entity)
                    .trigger(|e| MidiDisconnected { entity: e, reason });
            }
        }
    }
}

fn receive_midi_messages(mut inputs: Query<(&InputConnection, &mut MidiInputStream)>) {
    for (conn, mut stream) in &mut inputs {
        while let Ok(data) = conn.receiver.try_recv() {
            stream.messages.push(data);
        }
    }
}

fn send_midi_messages(mut outputs: Query<(&mut OutputConnection, &mut MidiOutputStream)>) {
    for (mut conn, mut stream) in &mut outputs {
        for msg in stream.outbox.drain(..) {
            if let Err(err) = conn.connection.send(&msg.msg) {
                warn!("failed to send MIDI message: {err}");
            }
        }
    }
}

fn on_midi_input_removed(event: On<Remove, MidiInput>, mut commands: Commands) {
    let entity = event.event_target();
    commands
        .entity(entity)
        .remove::<(InputConnection, MidiInputStream, MidiError)>();
}

fn on_midi_output_removed(event: On<Remove, MidiOutput>, mut commands: Commands) {
    let entity = event.event_target();
    commands
        .entity(entity)
        .remove::<(OutputConnection, MidiOutputStream, MidiError)>();
}
