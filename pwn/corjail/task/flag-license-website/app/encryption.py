from cryptography.hazmat.backends import default_backend
from cryptography.hazmat.primitives import padding
from cryptography.hazmat.primitives.ciphers import Cipher, algorithms, modes
from binascii import hexlify, unhexlify
import hashlib, base64

sha256 = lambda data : hashlib.sha256(data.encode()).hexdigest()

class AESDecryptor:


    def __init__(self, key, iv):

        self.key = key
        self.iv = iv
        self.cipher = Cipher(algorithms.AES(self.key), modes.CBC(self.iv), default_backend())


    def __unpad(self, data):

        unpadder = padding.PKCS7(algorithms.AES.block_size).unpadder()
        unpadded = unpadder.update(data) + unpadder.finalize()

        return unpadded


    def decrypt(self, data):
        
        try:
            decryptor = self.cipher.decryptor()
            plaintext = self.__unpad(decryptor.update(data)).decode()
        except Exception as e:
            print(f'[X] Decryption failed: {e}')
            return ''

        return plaintext
