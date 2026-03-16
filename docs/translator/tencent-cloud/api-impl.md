// 腾讯云API签名v3实现示例
// 本代码基于腾讯云API签名v3文档实现: https://cloud.tencent.com/document/product/213/30654
// 请严格按照文档说明使用，不建议随意修改签名相关代码
package main

import (
    "bytes"
    "crypto/hmac"
    "crypto/sha256"
    "encoding/hex"
    "os"
    "fmt"
    "log"
    "net/http"
    "strconv"
    "strings"
    "time"
)

func sha256hex(s string) string {
    b := sha256.Sum256([]byte(s))
    return hex.EncodeToString(b[:])
}

func hmacsha256(s, key string) string {
    hashed := hmac.New(sha256.New, []byte(key))
    hashed.Write([]byte(s))
    return string(hashed.Sum(nil))
}

func main() {
    service := "tmt"
    version := "2018-03-21"
    action := "TextTranslate"
    region := "ap-guangzhou"
    // 密钥信息从环境变量读取，需要提前在环境变量中设置 TENCENTCLOUD_SECRET_ID 和 TENCENTCLOUD_SECRET_KEY
    // 使用环境变量方式可以避免密钥硬编码在代码中，提高安全性
    // 生产环境建议使用更安全的密钥管理方案，如密钥管理系统(KMS)、容器密钥注入等
    // 请参见：https://cloud.tencent.com/document/product/1278/85305
    // 密钥可前往官网控制台 https://console.cloud.tencent.com/cam/capi 进行获取
    secretId := os.Getenv("TENCENTCLOUD_SECRET_ID")
    secretKey := os.Getenv("TENCENTCLOUD_SECRET_KEY")
    token := ""
    host := "tmt.tencentcloudapi.com"
    algorithm := "TC3-HMAC-SHA256"
    var timestamp = time.Now().Unix()

    // ************* 步骤 1：拼接规范请求串 *************
    httpRequestMethod := "POST"
    canonicalURI := "/"
    canonicalQueryString := ""
    contentType := "application/json; charset=utf-8"
    canonicalHeaders := fmt.Sprintf("content-type:%s\nhost:%s\nx-tc-action:%s\n",
        contentType, host, strings.ToLower(action))
    signedHeaders := "content-type;host;x-tc-action"
    payload := "{\"SourceText\":\"你好\",\"Source\":\"zh\",\"Target\":\"en\",\"ProjectId\":0}"
    hashedRequestPayload := sha256hex(payload)
    canonicalRequest := fmt.Sprintf("%s\n%s\n%s\n%s\n%s\n%s",
        httpRequestMethod,
        canonicalURI,
        canonicalQueryString,
        canonicalHeaders,
        signedHeaders,
        hashedRequestPayload)
    log.Println(canonicalRequest)

    // ************* 步骤 2：拼接待签名字符串 *************
    date := time.Unix(timestamp, 0).UTC().Format("2006-01-02")
    credentialScope := fmt.Sprintf("%s/%s/tc3_request", date, service)
    hashedCanonicalRequest := sha256hex(canonicalRequest)
    string2sign := fmt.Sprintf("%s\n%d\n%s\n%s",
        algorithm,
        timestamp,
        credentialScope,
        hashedCanonicalRequest)
    log.Println(string2sign)

    // ************* 步骤 3：计算签名 *************
    secretDate := hmacsha256(date, "TC3"+secretKey)
    secretService := hmacsha256(service, secretDate)
    secretSigning := hmacsha256("tc3_request", secretService)
    signature := hex.EncodeToString([]byte(hmacsha256(string2sign, secretSigning)))
    log.Println(signature)

    // ************* 步骤 4：拼接 Authorization *************
    authorization := fmt.Sprintf("%s Credential=%s/%s, SignedHeaders=%s, Signature=%s",
        algorithm,
        secretId,
        credentialScope,
        signedHeaders,
        signature)
    log.Println(authorization)

    // ************* 步骤 5：构造并发起请求 *************
    url := "https://" + host
    httpRequest, _ := http.NewRequest("POST", url, strings.NewReader(payload))
    httpRequest.Header = map[string][]string{
        "Host":           {host},
        "X-TC-Action":    {action},
        "X-TC-Version":   {version},
        "X-TC-Timestamp": {strconv.FormatInt(timestamp, 10)},
        "Content-Type":   {contentType},
        "Authorization":  {authorization},
    }
    if region != "" {
        httpRequest.Header["X-TC-Region"] = []string{region}
    }
    if token != "" {
        httpRequest.Header["X-TC-Token"] = []string{token}
    }
    httpClient := http.Client{}
    resp, err := httpClient.Do(httpRequest)
    if err != nil {
        log.Println(err)
        return
    }
    defer resp.Body.Close()
    body := &bytes.Buffer{}
    _, err = body.ReadFrom(resp.Body)
    if err != nil {
        log.Println(err)
        return
    }
    log.Println(body.String())
}