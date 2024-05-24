use bytes::BytesMut;
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};

// high watermark at 1MB
const HIGH_WATERMARK: u64 = 1024 * 1024;
const STEP_SIZE: u64 = HIGH_WATERMARK / 10;
#[allow(clippy::cast_possible_truncation)]
const BUF_SIZE: usize = STEP_SIZE as usize;

pub(super) async fn ack_reader(
    mut reader: impl AsyncRead + Unpin,
    mut writer: impl AsyncWrite + Unpin,
    mut target: impl AsyncWrite + Unpin,
) -> std::io::Result<()> {
    let mut buf = BytesMut::with_capacity(BUF_SIZE);
    let mut total_read = 0;
    let mut last_write = 0;
    loop {
        let count = reader.read_buf(&mut buf).await? as u64;
        if count == 0 {
            break;
        }
        target.write_all_buf(&mut buf).await?;
        total_read += count;
        if total_read - last_write >= STEP_SIZE {
            last_write = total_read;
            writer.write_u64(last_write).await?;
        }
    }
    target.flush().await?;
    target.shutdown().await?;
    writer.write_u64(total_read).await?;
    Ok(())
}

pub(super) async fn ack_writer(
    mut reader: impl AsyncRead + Unpin,
    mut writer: impl AsyncWrite + Unpin,
    mut target: impl AsyncRead + Unpin,
) -> std::io::Result<()> {
    let mut buf = BytesMut::with_capacity(BUF_SIZE);
    let mut total_write = 0;
    let mut last_read = 0;
    loop {
        if total_write - last_read >= HIGH_WATERMARK {
            writer.flush().await?;
            while total_write - last_read > STEP_SIZE * 2 {
                last_read = reader.read_u64().await?;
            }
        }
        let count = target.read_buf(&mut buf).await? as u64;
        if count == 0 {
            break;
        }
        writer.write_all_buf(&mut buf).await?;
        total_write += count;
    }
    writer.flush().await?;
    writer.shutdown().await?;
    while total_write - last_read > STEP_SIZE {
        last_read = reader.read_u64().await?;
    }
    reader.read_u64().await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use tokio::io::{duplex, split};

    use super::*;

    #[allow(clippy::cast_possible_truncation)]
    #[tokio::test]
    async fn test_duplex() {
        #[allow(clippy::cast_possible_truncation)]
        const TEST_BUFFER_SIZE: usize = HIGH_WATERMARK as usize * 4;

        let (alice, bob) = duplex(STEP_SIZE as usize * 2);
        let (alice_reader, alice_writer) = split(alice);
        let (bob_reader, bob_writer) = split(bob);

        let test_buffer = vec![0; TEST_BUFFER_SIZE];

        let alice_handle = async move {
            let (reader, mut writer) = duplex(STEP_SIZE as usize * 4);
            tokio::spawn(ack_writer(alice_reader, alice_writer, reader));
            for i in 0..1024 {
                writer.write_all(&test_buffer).await.unwrap();
                writer.write_u128(i).await.unwrap();
            }
        };
        let bob_handle = async move {
            let (mut reader, writer) = duplex(STEP_SIZE as usize * 4);
            tokio::spawn(ack_reader(bob_reader, bob_writer, writer));
            let mut buf = vec![0; TEST_BUFFER_SIZE];
            for i in 0..1024 {
                reader.read_exact(&mut buf).await.unwrap();
                let j = reader.read_u128().await.unwrap();
                assert_eq!(i, j);
            }
        };
        tokio::join!(alice_handle, bob_handle);
    }
}
