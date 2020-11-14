//
//  Base58Conversion.swift
//  Indy-demoTests
//
//  Created by Anastasia Tarasova on 06/06/2019.
//  Copyright © 2019 Hyperledger. All rights reserved.
//

import Foundation
import BigInt

public enum Base58Alphabet {
    public static let btc = [UInt8]("123456789ABCDEFGHJKLMNPQRSTUVWXYZabcdefghijkmnopqrstuvwxyz".utf8)
    public static let flickr = [UInt8]("123456789abcdefghijkmnopqrstuvwxyzABCDEFGHJKLMNPQRSTUVWXYZ".utf8)
}

@objc(Base58)
public class Base58: NSObject {
    
    @objc
    public static func encode(_ bytes: Data) -> String?
    {
        let alphabet: [UInt8] = Base58Alphabet.btc
        var x = BigUInt(bytes)
        let radix = BigUInt(alphabet.count)

        var answer = [UInt8]()
        answer.reserveCapacity(bytes.count)

        while x > 0 {
            let (quotient, modulus) = x.quotientAndRemainder(dividingBy: radix)
            answer.append(alphabet[Int(modulus)])
            x = quotient
        }

        let prefix = Array(bytes.prefix(while: {$0 == 0})).map { _ in alphabet[0] }
        answer.append(contentsOf: prefix)
        answer.reverse()

        return String(bytes: answer, encoding: .utf8)
    }

    @objc
    public static func decode(_ string: String) -> Data? {
        let alphabet: [UInt8] = Base58Alphabet.btc
        var answer = BigUInt(0)
        var j = BigUInt(1)
        let radix = BigUInt(alphabet.count)
        let byteString = [UInt8](string.utf8)

        for ch in byteString.reversed() {
            if let index = alphabet.firstIndex(of: ch) {
                answer = answer + (j * BigUInt(index))
                j *= radix
            } else {
                return nil
            }
        }

        let bytes = answer.serialize()
        return byteString.prefix(while: { i in i == alphabet[0]}) + bytes
    }
}
