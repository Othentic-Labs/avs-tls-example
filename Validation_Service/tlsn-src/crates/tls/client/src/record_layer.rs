use crate::{error::Error, Backend};
use tls_core::msgs::message::{OpaqueMessage, PlainMessage};

static SEQ_SOFT_LIMIT: u64 = 0xffff_ffff_ffff_0000u64;
static SEQ_HARD_LIMIT: u64 = 0xffff_ffff_ffff_fffeu64;

#[derive(PartialEq)]
enum DirectionState {
    /// No keying material.
    Invalid,

    /// Keying material present, but not yet in use.
    Prepared,

    /// Keying material in use.
    Active,
}

pub(crate) struct RecordLayer {
    write_seq: u64,
    read_seq: u64,
    encrypt_state: DirectionState,
    decrypt_state: DirectionState,

    // Message encrypted with other keys may be encountered, so failures
    // should be swallowed by the caller.  This struct tracks the amount
    // of message size this is allowed for.
    trial_decryption_len: Option<usize>,
}

impl RecordLayer {
    pub(crate) fn new() -> Self {
        Self {
            write_seq: 0,
            read_seq: 0,
            encrypt_state: DirectionState::Invalid,
            decrypt_state: DirectionState::Invalid,
            trial_decryption_len: None,
        }
    }

    pub(crate) fn is_encrypting(&self) -> bool {
        self.encrypt_state == DirectionState::Active
    }

    pub(crate) fn is_decrypting(&self) -> bool {
        self.decrypt_state == DirectionState::Active
    }

    pub(crate) fn doing_trial_decryption(&mut self, requested: usize) -> bool {
        match self
            .trial_decryption_len
            .and_then(|value| value.checked_sub(requested))
        {
            Some(remaining) => {
                self.trial_decryption_len = Some(remaining);
                true
            }
            _ => false,
        }
    }

    /// Prepare to use the given `MessageEncrypter` for future message
    /// encryption. It is not used until you call `start_encrypting`.
    pub(crate) fn prepare_message_encrypter(&mut self) {
        self.write_seq = 0;
        self.encrypt_state = DirectionState::Prepared;
    }

    /// Prepare to use the given `MessageDecrypter` for future message
    /// decryption. It is not used until you call `start_decrypting`.
    pub(crate) fn prepare_message_decrypter(&mut self) {
        self.read_seq = 0;
        self.decrypt_state = DirectionState::Prepared;
    }

    /// Start using the `MessageEncrypter` previously provided to the previous
    /// call to `prepare_message_encrypter`.
    pub(crate) fn start_encrypting(&mut self) {
        debug_assert!(self.encrypt_state == DirectionState::Prepared);
        self.encrypt_state = DirectionState::Active;
    }

    /// Start using the `MessageDecrypter` previously provided to the previous
    /// call to `prepare_message_decrypter`.
    pub(crate) fn start_decrypting(&mut self) {
        debug_assert!(self.decrypt_state == DirectionState::Prepared);
        self.decrypt_state = DirectionState::Active;
    }

    /// Set and start using the given `MessageEncrypter` for future outgoing
    /// message encryption.
    pub(crate) fn set_message_encrypter(&mut self) {
        self.prepare_message_encrypter();
        self.start_encrypting();
    }

    /// Set and start using the given `MessageDecrypter` for future incoming
    /// message decryption.
    pub(crate) fn set_message_decrypter(&mut self) {
        self.prepare_message_decrypter();
        self.start_decrypting();
        self.trial_decryption_len = None;
    }

    /// Set and start using the given `MessageDecrypter` for future incoming
    /// message decryption, and enable "trial decryption" mode for when TLS1.3
    /// 0-RTT is attempted but rejected by the server.
    pub(crate) fn set_message_decrypter_with_trial_decryption(&mut self, max_length: usize) {
        self.prepare_message_decrypter();
        self.start_decrypting();
        self.trial_decryption_len = Some(max_length);
    }

    pub(crate) fn finish_trial_decryption(&mut self) {
        self.trial_decryption_len = None;
    }

    /// Return true if the peer appears to getting close to encrypting
    /// too many messages with this key.
    ///
    /// Perhaps if we send an alert well before their counter wraps, a
    /// buggy peer won't make a terrible mistake here?
    ///
    /// Note that there's no reason to refuse to decrypt: the security
    /// failure has already happened.
    pub(crate) fn wants_close_before_decrypt(&self) -> bool {
        self.read_seq == SEQ_SOFT_LIMIT
    }

    /// Return true if we are getting close to encrypting too many
    /// messages with our encryption key.
    pub(crate) fn wants_close_before_encrypt(&self) -> bool {
        self.write_seq == SEQ_SOFT_LIMIT
    }

    /// Return true if we outright refuse to do anything with the
    /// encryption key.
    pub(crate) fn encrypt_exhausted(&self) -> bool {
        self.write_seq >= SEQ_HARD_LIMIT
    }

    /// Decrypt a TLS message.
    ///
    /// `encr` is a decoded message allegedly received from the peer.
    /// If it can be decrypted, its decryption is returned.  Otherwise,
    /// an error is returned.
    pub(crate) async fn decrypt_incoming(
        &mut self,
        backend: &mut dyn Backend,
        encr: OpaqueMessage,
    ) -> Result<(), Error> {
        debug_assert!(self.is_decrypting());
        backend.push_incoming(encr).await?;
        self.read_seq += 1;
        Ok(())
    }

    /// Encrypt a TLS message.
    ///
    /// `plain` is a TLS message we'd like to send.  This function
    /// panics if the requisite keying material hasn't been established yet.
    pub(crate) async fn encrypt_outgoing(
        &mut self,
        backend: &mut dyn Backend,
        plain: PlainMessage,
    ) -> Result<(), Error> {
        debug_assert!(self.encrypt_state == DirectionState::Active);
        assert!(!self.encrypt_exhausted());
        backend.push_outgoing(plain).await?;
        self.write_seq += 1;
        Ok(())
    }
}
