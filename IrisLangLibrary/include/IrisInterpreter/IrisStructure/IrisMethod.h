#ifndef _H_IRISMETHOD_
#define _H_IRISMETHOD_

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

#ifdef IR_USE_MEM_POOL
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
	typedef IrisValue(*IrisNativeFunction)(IrisValue&, IIrisValues*, IIrisValues*, IIrisContextEnvironment*);

public:
	struct UserFunction {
		list<IrisInternString> m_lsParameters;
		IrisInternString m_strVariableParameter = "";

		IrisCodeSegment m_icsBlockCodes;
		IrisCodeSegment m_icsWithBlockCodes;
		IrisCodeSegment m_icsWithoutBlockCodes;

		unsigned int m_dwIndex = 0;

		UserFunction() = default;
		~UserFunction() = default;
	};

private:
	IrisInternString m_strMethodName = "";
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
	IrisContextEnvironment* _CreateContextEnvironment(IrisObject* pCaller, IrisValues* pParameters, IrisContextEnvironment* pContextEnvrioment, bool& bIsGetNew);

public:
	IrisMethod(const IrisInternString& strMethodName, IrisNativeFunction pfNativeFunction, int nParameterAmount, bool bIsWithVariableParameter, MethodAuthority eAuthority = MethodAuthority::Everyone);
	IrisMethod(const IrisInternString& strMethodName, UserFunction* pUserFunction, MethodAuthority eAuthority = MethodAuthority::Everyone);
	IrisMethod(const IrisInternString& strMethodName, UserFunction* pUserFunction, MethodType eType, MethodAuthority eAuthority = MethodAuthority::Everyone);

	void ResetObject();

	void SetMethodAuthority(MethodAuthority eAuthority);

	/* Native Function的定义
	IrisValue FunctionName(IrisValue ivObject, IrisValue ivParam1, IrisValue ivParam2, ...);
	*/
	// 按照类型的不同分别调用不同的函数（直接调用 or 解释运行）
	IrisValue Call(IrisValue& ivObject, IrisContextEnvironment* pContexEnvironment, IrisValues* pParameters = nullptr, unsigned int nLineNumber = -1, int nBelongingFileIndex = -1);
	IrisValue CallMainMethod(IrisValues* pParameters = nullptr, unsigned int nLineNumber = -1, int nBelongingFileIndex = -1);

	~IrisMethod();

	//Getter Setter

	const IrisInternString& GetMethodName();

	void SetMethodName(const IrisInternString& strMethodName);

	MethodAuthority GetAuthority();

	inline bool IsWithVariableParameter() { return m_bIsWithVariableParameter; }
	inline unsigned int GetParameterAmount() { return m_nParameterAmount; }
	inline IIrisObject* GetMethodObject() { return m_pMethodObject; }
	friend class IrisGC;
};

#endif