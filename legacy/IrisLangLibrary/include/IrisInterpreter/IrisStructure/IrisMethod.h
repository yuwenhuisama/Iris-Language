#ifndef _H_IRISMETHOD_
#define _H_IRISMETHOD_

#include "IrisCompileConfigure.h"

#include "IrisUnil/IrisValue.h"
#include "IrisComponents/IrisVirtualCodeStructures.h"
#include "IrisUnil/IrisCodeSegment.h"
#include "IrisUnil/IrisMemoryPool/IrisObjectMemoryPoolInterface.h"
#include "IrisUnil/IrisMemoryPool/IrisMemoryPoolDefines.h"
#include "IrisUnil/IrisInternString.h"

#include <string>
#include <vector>
#include <list>
using namespace std;

class IrisObject;
class IrisBlock;
class IrisFunctionHeader;
class IrisContextEnvironment;
class IrisValues;

class IIrisValues;
class IIrisContextEnvironment;
class IIrisThreadInfo;
class IrisThreadInfo;

#if IR_USE_MEM_POOL
class IrisMethod : public IrisObjectMemoryPoolInterface<IrisMethod, POOLID_IrisMethod> 
#else
class IrisMethod
#endif
{
public:
	enum class MethodType {
		NativeMethod = 0,
		UserMethod,
		GetterMethod,
		SetterMethod,
	};

	enum class MethodAuthority {
		Everyone = 0,
		Relative,
		Personal,
	};

public:
	typedef IrisValue(*IrisNativeFunction)(const IrisValue&, IIrisValues*, IIrisValues*, IIrisContextEnvironment*, IIrisThreadInfo*);

public:
	struct UserFunction {
#if IR_USE_STL_STRING
		list<string> m_lsParameters;
		string m_strVariableParameter = "";
#else
		list<IrisInternString> m_lsParameters;
		IrisInternString m_strVariableParameter = "";
#endif // IR_USE_STL_STRING

		IrisCodeSegment m_icsBlockCodes;
		IrisCodeSegment m_icsWithBlockCodes;
		IrisCodeSegment m_icsWithoutBlockCodes;

		unsigned int m_dwIndex = 0;

		UserFunction() = default;
		~UserFunction() = default;
	};

private:
#if IR_USE_STL_STRING
	string m_strMethodName = "";
#else
	IrisInternString m_strMethodName = "";
#endif // IR_USE_STL_STRING
	bool m_bIsWithVariableParameter = false;
	MethodType m_eMethodType = MethodType::NativeMethod;
	unsigned int m_nParameterAmount = 0;
	union {
		IrisNativeFunction m_pfNativeFunction = nullptr;
		UserFunction* m_pUserFunction;
	} m_uFunction;

	MethodAuthority m_eAuthority = MethodAuthority::Everyone;

	IIrisObject* m_pMethodObject = nullptr;

	bool _ParameterCheck(IrisValues* pParameters);
	IrisContextEnvironment* _CreateContextEnvironment(IrisObject* pCaller, IrisValues* pParameters, IrisContextEnvironment* pContextEnvrioment, IrisThreadInfo* pThreadInfo, bool& bIsGetNew);

public:

#if IR_USE_STL_STRING
	IrisMethod(const string& strMethodName, IrisNativeFunction pfNativeFunction, int nParameterAmount, IrisThreadInfo* pThreadInfo, bool bIsWithVariableParameter, MethodAuthority eAuthority = MethodAuthority::Everyone);
	IrisMethod(const string& strMethodName, UserFunction* pUserFunction, IrisThreadInfo* pThreadInfo, MethodAuthority eAuthority = MethodAuthority::Everyone);
	IrisMethod(const string& strMethodName, UserFunction* pUserFunction, MethodType eType, IrisThreadInfo* pThreadInfo, MethodAuthority eAuthority = MethodAuthority::Everyone);
#else
	IrisMethod(const IrisInternString& strMethodName, IrisNativeFunction pfNativeFunction, int nParameterAmount, bool bIsWithVariableParameter, MethodAuthority eAuthority = MethodAuthority::Everyone);
	IrisMethod(const IrisInternString& strMethodName, UserFunction* pUserFunction, MethodAuthority eAuthority = MethodAuthority::Everyone);
	IrisMethod(const IrisInternString& strMethodName, UserFunction* pUserFunction, MethodType eType, MethodAuthority eAuthority = MethodAuthority::Everyone);
#endif // IR_USE_STL_STRING

	void ResetObject();

	void SetMethodAuthority(MethodAuthority eAuthority);

	/* Native Function的定义
	IrisValue FunctionName(IrisValue ivObject, IrisValue ivParam1, IrisValue ivParam2, ...);
	*/
	// 按照类型的不同分别调用不同的函数（直接调用 or 解释运行）
	IrisValue Call(IrisValue& ivObject, IrisValues* pParameters, IrisContextEnvironment* pContexEnvironment, IrisThreadInfo* pThreadInfo);
	IrisValue CallMainMethod(IrisValues* pParameters, IrisThreadInfo* pThreadInfo);

	~IrisMethod();

	//Getter Setter


#if IR_USE_STL_STRING
	const string& GetMethodName();
	void SetMethodName(const string& strMethodName);
#else
	const IrisInternString& GetMethodName();
	void SetMethodName(const IrisInternString& strMethodName);
#endif // IR_USE_STL_STRING


	MethodAuthority GetAuthority();

	inline bool IsWithVariableParameter() { return m_bIsWithVariableParameter; }
	inline unsigned int GetParameterAmount() { return m_nParameterAmount; }
	inline IIrisObject* GetMethodObject() { return m_pMethodObject; }
	friend class IrisGC;
};

#endif